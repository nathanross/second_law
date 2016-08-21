extern crate tempdir;

use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::path::PathBuf;
use std::sync::Arc;
use self::tempdir::TempDir;

use super::atpath::AtPath;
use super::ucommand::UCommand;
use super::common::recursive_copy;

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

#[cfg(windows)]
static PROGNAME: &'static str = "uutils.exe";
#[cfg(not(windows))]
static PROGNAME: &'static str = "uutils";

static TESTS_DIR: &'static str = "tests";
static FIXTURES_DIR: &'static str = "fixtures";

/// An environment for running a single uutils test case, serves three functions:
/// 1. centralizes logic for locating the uutils binary and calling the utility
/// 2. provides a temporary directory for the test case
/// 3. copies over fixtures for the utility to the temporary directory
pub struct Scene<'ts> {
    bin_path: PathBuf,
    util_name: &'ts OsStr,
    tmpd: Arc<TempDir>,
}

impl<'ts> Scene<'ts> {
    pub fn new(util_name: &'ts OsStr) -> Scene<'ts> {
        let tmpd = Arc::new(TempDir::new("uutils").unwrap());
        let ts = Scene {
            bin_path: {
                // Instead of hardcoding the path relative to the current
                // directory, use Cargo's OUT_DIR to find path to executable.
                // This allows tests to be run using profiles other than debug.
                let target_dir = path_concat!(env::var("OUT_DIR").unwrap(), "..", "..", "..", PROGNAME);
                PathBuf::from(AtPath::from_path(&target_dir).root_dir_resolved())
            },
            util_name: util_name.as_ref(),
            tmpd: tmpd,
        };
        let mut fixture_path_builder = env::current_dir().unwrap();
        fixture_path_builder.push(TESTS_DIR);
        fixture_path_builder.push(FIXTURES_DIR);
        fixture_path_builder.push(util_name);
        match fs::metadata(&fixture_path_builder) {
            Ok(m) => if m.is_dir() {
                recursive_copy(&fixture_path_builder, ts.tmpd.as_ref().path()).unwrap();
            },
            Err(_) => {}
        }
        ts
    }
    
    pub fn fixtures<'tmpd>(&'tmpd self) -> AtPath<'tmpd> {
        AtPath::from_path(self.tmpd.as_ref().path())
    }
    
    pub fn fixtures_owned<'tmpd>(&self) -> AtPath<'tmpd> {
        AtPath::from_path_owned(self.tmpd.as_ref().path().to_owned())
    }    
    
    pub fn ucmd(&self) -> UCommand {
        let mut cmd = self.cmd(&self.bin_path);
        cmd.arg(&self.util_name);
        cmd
    }

    pub fn cmd<S: AsRef<OsStr>>(&self, bin: S) -> UCommand {
        UCommand::new_from_tmp(bin, self.tmpd.clone(), true)
    }

    // different names are used rather than an argument
    // because the need to keep the environment is exceedingly rare.
    pub fn ucmd_keepenv(&self) -> UCommand {
        let mut cmd = self.cmd_keepenv(&self.bin_path);
        cmd.arg(&self.util_name);
        cmd
    }

    pub fn cmd_keepenv<S: AsRef<OsStr>>(&self, bin: S) -> UCommand {
        UCommand::new_from_tmp(bin, self.tmpd.clone(), false)
    }
}
