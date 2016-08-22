extern crate tempdir;

use std::ffi::OsStr;
use std::sync::Arc;

use super::atpath::AtPath;
use super::settings::SceneSettings;

pub fn read_scenario_fixture<S: AsRef<OsStr>>(settings: &Arc<SceneSettings>, file_rel_path: S) -> String {
    let tmpdir_path = settings.as_ref().tmpd.path();
    AtPath::from_path(tmpdir_path).read(file_rel_path.as_ref().to_str().unwrap())
}
