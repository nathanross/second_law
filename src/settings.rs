extern crate tempdir;

use std::ffi::OsString;
use std::path::PathBuf;
use self::tempdir::TempDir;

pub struct SceneSettings {
    pub debug_bin_path: PathBuf,
    pub repo_fixtures_path: PathBuf,
    pub subcmd_args: Vec<OsString>,
    pub tmpd: TempDir,
}
