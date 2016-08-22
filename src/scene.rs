extern crate tempdir;

use std::env;
use std::ffi::{OsStr,OsString};
use std::fs::{self};
use std::path::{Path,PathBuf};
use std::sync::Arc;
use self::tempdir::TempDir;

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
static BIN_SUBPATH_MAX_ONCE: &'static str = "the bin subpath can only be set once per scene";
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

    pub fn new() -> Scene {
        Scene {
            builder : Some(SceneBuilder {
                debug_bin_subpath: None,
                fixtroot_fixture_subpath: None,
                repo_fixtroot_subpath: None,
                subcmd_args: None,
                multicall: None
            }),
            setting : None
        }
    }

    pub fn bin_subpath<P: AsRef<Path>>(&mut self, subpath : P) {
        if let Some(ref mut builder) = self.builder {
            if builder.debug_bin_subpath.is_some() {
                panic!(BIN_SUBPATH_MAX_ONCE);
            }
            builder.debug_bin_subpath = Some(PathBuf::from(subpath.as_ref()));
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
    }

    pub fn fixtures_root<P: AsRef<Path>>(&mut self, root : P) {
        if let Some(ref mut builder) = self.builder {
            if builder.repo_fixtroot_subpath.is_some() {
                panic!(ROOT_CALLED_MAX_ONCE);
            }
            builder.repo_fixtroot_subpath = Some(PathBuf::from(root.as_ref()));
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
    }

    pub fn fixtures_subdir<P: AsRef<Path>>(&mut self, subdirs : P) {
        if let Some(ref mut builder) = self.builder {
            if let Some(ref mut fixtroot_fixture_subpath) = builder.fixtroot_fixture_subpath {
                fixtroot_fixture_subpath.push(subdirs.as_ref());
            } else {
                builder.repo_fixtroot_subpath = Some(PathBuf::from(subdirs.as_ref()));
            }            
        } else {
            panic!(ALREADY_INSTANTIATED);
        }
    }

    pub fn multicall<S: AsRef<OsStr>>(&mut self, subcmd_arg : S) {
        self.fixtures_subdir(Path::new(subcmd_arg.as_ref()));
        self.subcmd_arg(subcmd_arg.as_ref());
    }

    pub fn subcmd_arg<S: AsRef<OsStr>>(&mut self, added_arg : S) {
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
    }

    pub fn subcmd_args<S: AsRef<OsStr>>(&mut self, added_args : &[S]) {
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
    }
    
    pub fn ucmd(&mut self) -> UCommand {
        let settings = self.setting.as_ref().unwrap().clone();
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
        let settings = self.setting.as_ref().unwrap().clone();
        let mut cmd = self.cmd_keepenv(&settings.as_ref().debug_bin_path);
        cmd.args(&settings.as_ref().subcmd_args);
        cmd
    }

    pub fn cmd_keepenv<S: AsRef<OsStr>>(&mut self, bin: S) -> UCommand {
        let setting = self.cloned_setting();
        let curdir : Option<&OsStr>  = None;
        UCommand::new(bin, setting, false, curdir)
    }
    
    fn cloned_setting(&mut self) -> Arc<SceneSettings> {
        if let Some(ref setting) = self.setting {
            setting.clone()
        } else {
            {
                let builder = self.builder.as_ref().unwrap();
                let result = Arc::new(self.generate_setting(&builder));
                {
                    let fixtures_path = &result.as_ref().repo_fixtures_path;
                    match fs::metadata(fixtures_path) {
                        Ok(m) => if m.is_dir() {
                            recursive_copy(fixtures_path, result.as_ref().tmpd.path()).unwrap();
                        },
                        Err(_) => {
                            panic!("error copying to fixtures directory {}. Are you sure it exists?", fixtures_path.to_str().unwrap());
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
                let mut target_dir = PathBuf::from(path_concat!(
                    env::var("OUT_DIR").unwrap(), "..", "..", ".."));
                target_dir.push(
                    if let Some(ref bin_subpath) = builder.debug_bin_subpath {
                        bin_subpath.clone()
                    } else {
                        PathBuf::from(env!("CARGO_PKG_NAME"))
                    }
                );
                target_dir
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
                repo_fixtures_subpath
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
            tmpd: TempDir::new("second_law").unwrap()
        }
    }
}

