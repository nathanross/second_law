
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_file;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;

use super::log_info;

/// Object-oriented path struct that represents and operates on
/// paths relative to the directory it was constructed for.
pub struct AtPath<'at> {
    pub subdir: Cow<'at, Path>,
}

impl<'at> AtPath<'at> {
    pub fn from_osstr(subdir: &'at OsStr) -> AtPath<'at> {
        AtPath { subdir: Cow::Borrowed(Path::new(subdir)) }
    }

    pub fn from_path(subdir: &'at Path) -> AtPath<'at> {
        AtPath { subdir: Cow::Borrowed(subdir) }
    }

    pub fn from_path_owned(subdir: PathBuf) -> AtPath<'at> {
        AtPath { subdir : Cow::Owned(subdir) }
    }
    
    pub fn as_string(&self) -> String {
        self.subdir.to_str().unwrap().to_owned()
    }

    pub fn plus(&self, name: &str) -> PathBuf {
        let mut pathbuf = PathBuf::from(self.subdir.as_ref());
        pathbuf.push(name);
        pathbuf
    }

    pub fn plus_as_string(&self, name: &str) -> String {
        String::from(self.plus(name).to_str().unwrap())
    }

    fn minus(&self, name: &str) -> PathBuf {
        let prefixed = PathBuf::from(name);
        if prefixed.starts_with(&self.subdir) {
            let mut unprefixed = PathBuf::new();
            for component in prefixed.components()
                                     .skip(self.subdir.components().count()) {
                unprefixed.push(component.as_ref().to_str().unwrap());
            }
            unprefixed
        } else {
            prefixed
        }
    }

    pub fn minus_as_string(&self, name: &str) -> String {
        String::from(self.minus(name).to_str().unwrap())
    }

    pub fn open(&self, name: &str) -> File {
        log_info("open", self.plus_as_string(name));
        File::open(self.plus(name)).unwrap()
    }

    pub fn read(&self, name: &str) -> String {
        let mut f = self.open(name);
        let mut contents = String::new();
        let _ = f.read_to_string(&mut contents);
        contents
    }

    pub fn write(&self, name: &str, contents: &str) {
        let mut f = self.open(name);
        let _ = f.write(contents.as_bytes());
    }

    pub fn append(&self, name: &str, contents: &str) {
        log_info("open(append)", self.plus_as_string(name));
        let mut f = OpenOptions::new().write(true).append(true).open(self.plus(name)).unwrap();
        let _ = f.write(contents.as_bytes());
    }

    pub fn mkdir(&self, dir: &str) {
        log_info("mkdir", self.plus_as_string(dir));
        fs::create_dir(&self.plus(dir)).unwrap();
    }
    pub fn mkdir_all(&self, dir: &str) {
        log_info("mkdir_all", self.plus_as_string(dir));
        fs::create_dir_all(self.plus(dir)).unwrap();
    }

    pub fn make_file(&self, name: &str) -> File {
        match File::create(&self.plus(name)) {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        }
    }

    pub fn touch(&self, file: &str) {
        log_info("touch", self.plus_as_string(file));
        File::create(&self.plus(file)).unwrap();
    }

    pub fn symlink(&self, src: &str, dst: &str) {
        log_info("symlink",
                 &format!("{},{}", self.plus_as_string(src), self.plus_as_string(dst)));
        symlink_file(&self.plus(src), &self.plus(dst)).unwrap();
    }

    pub fn is_symlink(&self, path: &str) -> bool {
        log_info("is_symlink", self.plus_as_string(path));
        match fs::symlink_metadata(&self.plus(path)) {
            Ok(m) => m.file_type().is_symlink(),
            Err(_) => false,
        }
    }

    pub fn resolve_link(&self, path: &str) -> String {
        log_info("resolve_link", self.plus_as_string(path));
        match fs::read_link(&self.plus(path)) {
            Ok(p) => {
                self.minus_as_string(p.to_str().unwrap())
            }
            Err(_) => "".to_string(),
        }
    }

    pub fn metadata(&self, path: &str) -> fs::Metadata {
        match fs::metadata(&self.plus(path)) {
            Ok(m) => m,
            Err(e) => panic!("{}", e),
        }
    }

    pub fn file_exists(&self, path: &str) -> bool {
        match fs::metadata(&self.plus(path)) {
            Ok(m) => m.is_file(),
            Err(_) => false,
        }
    }

    pub fn dir_exists(&self, path: &str) -> bool {
        match fs::metadata(&self.plus(path)) {
            Ok(m) => m.is_dir(),
            Err(_) => false,
        }
    }

    pub fn cleanup(&self, path: &'static str) {
        let p = &self.plus(path);
        match fs::metadata(p) {
            Ok(m) => if m.is_file() {
                fs::remove_file(&p).unwrap();
            } else {
                fs::remove_dir(&p).unwrap();
            },
            Err(_) => {}
        }
    }

    pub fn root_dir(&self) -> String {
        log_info("current_directory", "");
        self.subdir.to_str().unwrap().to_owned()
    }

    pub fn root_dir_resolved(&self) -> String {
        log_info("current_directory_resolved", "");
        let s = self.subdir.canonicalize().unwrap().to_str().unwrap().to_owned();

        // Due to canonicalize()'s use of GetFinalPathNameByHandleW() on Windows, the resolved path
        // starts with '\\?\' to extend the limit of a given path to 32,767 wide characters.
        //
        // To address this issue, we remove this prepended string if available.
        //
        // Source:
        // http://stackoverflow.com/questions/31439011/getfinalpathnamebyhandle-without-prepended
        let prefix = "\\\\?\\";
        if s.starts_with(prefix) {
            String::from(&s[prefix.len()..])
        } else {
            s
        }
    }
}
