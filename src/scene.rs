extern crate tempdir;

use std::env;
use std::ffi::{OsStr,OsString};
use std::fs::{self};
use std::path::{Path,PathBuf};
use std::sync::Arc;
use self::tempdir::TempDir;

use super::atpath::AtPath;
use super::ucommand::UCommand;
use super::common::recursive_copy;
use super::settings::SceneSettings;

//#[macro_export]
macro_rules! path_concat {
    ($e:expr, ..$n:expr) => {{
        use std::path::PathBuf;
        let n = $n;
        let mut pb = PathBuf::new();
        for _ in 0..n {
            pb.push($e);
        }
        pb
    }};
    ($($e:expr),*) => {{
        use std::path::PathBuf;
        let mut pb = PathBuf::new();
        $(
            pb.push($e);
        )*
        pb
    }};
}

static DEFAULT_FIXTURES_ROOT: &'static str = "tests/fixtures";
static ROOT_CALLED_MAX_ONCE: &'static str = "the fixture root can only be set once. To add subdirectories in multiple steps, use .fixtures_subdir(:&Path)";
static ALREADY_INSTANTIATED: &'static str = "configuration of a scene must be done before the first call to its .cmd() or .ucmd()";

// why not lifetimes? design choices explanation at the end of this source

struct SceneBuilder {
    pub debug_bin_subpath: Option<PathBuf>,
    pub fixtroot_fixture_subpath: Option<PathBuf>,
    pub repo_fixtroot_subpath: Option<PathBuf>,
    pub subcmd_args: Option<Vec<OsString>>,
    pub multicall: Option<OsString>,
}

/// An environment for running a single uutils test case, serves three functions:
/// 1. centralizes logic for locating the uutils binary and calling the utility
/// 2. provides a temporary directory for the test case
/// 3. copies over fixtures for the utility to the temporary directory

// theres probably a better way to do this using
// enum rather than option.
pub struct Scene {
    builder : Option<SceneBuilder>,
    setting : Option<Arc<SceneSettings>>
}
impl Scene {

    pub fn new<P: AsRef<Path>>(bin_subpath : P) -> Scene {
        Scene {
            builder : Some(SceneBuilder {
                debug_bin_subpath: Some(PathBuf::from(bin_subpath.as_ref())),
                fixtroot_fixture_subpath: None,
                repo_fixtroot_subpath: None,
                subcmd_args: None,
                multicall: None
            }),
            setting : None
        }
    }

    pub fn fixtures_root<'a, P: AsRef<Path>>(&'a mut self, root : P) -> &'a Scene {
        if let Some(ref mut builder) = self.builder {
            if builder.repo_fixtroot_subpath.is_some() {
                panic!(ROOT_CALLED_MAX_ONCE);
            }
            builder.repo_fixtroot_subpath = Some(PathBuf::from(root.as_ref()));
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
        self
    }

    pub fn fixtures_subdir<'a, P: AsRef<Path>>(&'a mut self, subdirs : P) -> &'a Scene {
        if let Some(ref mut builder) = self.builder {
            if let Some(ref mut fixtroot_fixture_subpath) = builder.fixtroot_fixture_subpath {
                fixtroot_fixture_subpath.push(subdirs.as_ref());
            } else {
                builder.repo_fixtroot_subpath = Some(PathBuf::from(subdirs.as_ref()));
            }            
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
        self
    }

    pub fn multicall<'a, S: AsRef<OsStr>>(&'a mut self, subcmd_arg : S) -> &'a Scene {
        self.fixtures_subdir(Path::new(subcmd_arg.as_ref()));
        self.subcmd_arg(subcmd_arg.as_ref());
        self
    }

    pub fn subcmd_arg<'a, S: AsRef<OsStr>>(&'a mut self, added_arg : S) -> &'a Scene {
        if let Some(ref mut builder) = self.builder {
            if let Some(ref mut current_value) = builder.subcmd_args {
                current_value.push(OsString::from(added_arg.as_ref()));
            } else {
                let mut result : Vec<OsString> = Vec::new();
                result.push(OsString::from(added_arg.as_ref()));
                builder.subcmd_args = Some(result);
            }         
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
        self
    }

    pub fn subcmd_args<'a, S: AsRef<OsStr>>(&'a mut self, added_args : &[S]) -> &'a Scene {
        if let Some(ref mut builder) = self.builder {
            if let Some(ref mut current_value) = builder.subcmd_args {
                for val in added_args.iter() {
                    current_value.push(OsString::from(val.as_ref()));
                }
            } else {
                let mut result : Vec<OsString> = Vec::new();
                for val in added_args.iter() {
                    result.push(OsString::from(val.as_ref()));
                }
                builder.subcmd_args = Some(result);
            }            
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
        self
    }
    
    pub fn ucmd(&mut self) -> UCommand {
        let settings = self.cloned_setting();
        let mut cmd = self.cmd(&settings.as_ref().debug_bin_path);
        cmd.args(&settings.as_ref().subcmd_args);
        cmd
    }

    pub fn cmd<S: AsRef<OsStr>>(&mut self, bin: S) -> UCommand {
        let setting = self.cloned_setting();
        let curdir : Option<&OsStr>  = None;
        UCommand::new(bin, setting, true, curdir)
    }

    // different names are used rather than an argument
    // because the need to keep the environment is exceedingly rare.
    pub fn ucmd_keepenv(&mut self) -> UCommand {
        let settings = self.cloned_setting();
        let mut cmd = self.cmd_keepenv(&settings.as_ref().debug_bin_path);
        cmd.args(&settings.as_ref().subcmd_args);
        cmd
    }

    pub fn cmd_keepenv<S: AsRef<OsStr>>(&mut self, bin: S) -> UCommand {
        let setting = self.cloned_setting();
        let curdir : Option<&OsStr>  = None;
        UCommand::new(bin, setting, false, curdir)
    }

    pub fn working_dir(&mut self) -> AtPath {
        let setting = self.cloned_setting();
        AtPath::from_scene_settings(setting)
    }
    
    fn cloned_setting(&mut self) -> Arc<SceneSettings> {
        if let Some(ref setting) = self.setting {
            setting.clone()
        } else {
            {
                let builder = self.builder.as_ref().expect("tried to retrieve builder but could not");
                let result = Arc::new(self.generate_setting(&builder));
                {
                    match &result.as_ref().repo_fixtures_path {
                        &None => {},
                        &Some(ref fixtures_path) => {
                            match fs::metadata(&fixtures_path) {
                                Ok(m) => if m.is_dir() {
                                    recursive_copy(&fixtures_path, result.as_ref().tmpd.path()).expect("tried to recursively copy fixtures to tmp dir but failed");
                                },
                                Err(_) => {
                                    panic!("error copying to fixtures directory {}. Are you sure it exists?", fixtures_path.to_str().expect("tried to get provided fixtures path as a string but failed"));
                                }
                            }
                        }
                    }
                }
                self.setting = Some(result);
            }
            
            self.builder = None;
            self.setting.as_ref().unwrap().clone()
        }
    }

    fn generate_setting(&self, builder: &SceneBuilder) -> SceneSettings {
        SceneSettings {
            debug_bin_path: {
                // Instead of hardcoding the path relative to the current
                // directory, use Cargo's OUT_DIR to find path to executable.
                // This allows tests to be run using profiles other than debug.
                let environment_string = env::var("LD_LIBRARY_PATH").expect("this environment variable is depended upon to find the executable path");
                let library_path_split : Vec<&str> = environment_string.split(":").collect();
                println!("{}", library_path_split[0]);
                let mut target_dir = PathBuf::from(library_path_split[0]);
                target_dir.push(
                    builder.debug_bin_subpath.as_ref().unwrap().clone()
                );
                PathBuf::from(AtPath::from_path_owned(target_dir).root_dir_resolved().unwrap())
            },
            repo_fixtures_path: {
                let mut repo_fixtures_subpath = {
                    if let Some(ref repo_fixtroot_subpath) = builder.repo_fixtroot_subpath {
                        PathBuf::from(repo_fixtroot_subpath)
                    } else {
                        PathBuf::from(DEFAULT_FIXTURES_ROOT)
                    }
                };
                if let Some(ref subcommand_name) = builder.multicall {
                    repo_fixtures_subpath.push(subcommand_name);
                };
                if let Some(ref fixtroot_fixture_subpath) = builder.fixtroot_fixture_subpath {
                    repo_fixtures_subpath.push(fixtroot_fixture_subpath);
                };
                AtPath::from_path_owned(repo_fixtures_subpath).root_dir_resolved().map(|x| PathBuf::from(x))
            },
            subcmd_args: {
                let mut result = if let Some(ref subcmd_args) = builder.subcmd_args {
                    subcmd_args.clone()
                } else {
                    Vec::new()
                };
                if let Some(ref subcommand_name) = builder.multicall {
                    result.insert(0, subcommand_name.clone());
                };
                result
            },
            tmpd: TempDir::new("second_law").expect("tried to create a temporary directory but failed")
        }
    }
}

