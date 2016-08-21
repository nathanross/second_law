

mod atpath;
mod fixtures;
mod cmdresult;
mod ucommand;
mod scenario;

use std::fs::{self};
use std::io::{Read, Result};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread::sleep;
use std::time::Duration;

pub use atpath::AtPath;
pub use ucommand::UCommand;
pub use scenario::Scenario;
pub use cmdresult::CmdResult;

pub fn repeat_str(s: &str, n: u32) -> String {
    let mut repeated = String::new();
    for _ in 0..n {
        repeated.push_str(s);
    }
    repeated
}


pub fn log_info<T: AsRef<str>, U: AsRef<str>>(msg: T, par: U) {
    println!("{}: {}", msg.as_ref(), par.as_ref());
}

pub fn recursive_copy(src: &Path, dest: &Path) -> Result<()> {
    if try!(fs::metadata(src)).is_dir() {
        for entry in try!(fs::read_dir(src)) {
            let entry = try!(entry);
            let mut new_dest = PathBuf::from(dest);
            new_dest.push(entry.file_name());
            if try!(fs::metadata(entry.path())).is_dir() {
                try!(fs::create_dir(&new_dest));
                try!(recursive_copy(&entry.path(), &new_dest));
            } else {
                try!(fs::copy(&entry.path(), new_dest));
            }
        }
    }
    Ok(())
}

pub fn get_root_path() -> &'static str {
    if cfg!(windows) {
        "C:\\"
    } else {
        "/"
    }
}



pub fn read_size(child: &mut Child, size: usize) -> String {
    let mut output = Vec::new();
    output.resize(size, 0);
    sleep(Duration::from_secs(1));
    child.stdout.as_mut().unwrap().read(output.as_mut_slice()).unwrap();
    String::from_utf8(output).unwrap()
}
