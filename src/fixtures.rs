extern crate tempdir;

use std::ffi::OsStr;
use std::sync::Arc;
use self::tempdir::TempDir;

use super::atpath::AtPath;

pub fn read_scenario_fixture<S: AsRef<OsStr>>(tmpd: &Option<Arc<TempDir>>, file_rel_path: S) -> String {
    let tmpdir_path = tmpd.as_ref().unwrap().as_ref().path();
    AtPath::from_path(tmpdir_path).read(file_rel_path.as_ref().to_str().unwrap())
}
