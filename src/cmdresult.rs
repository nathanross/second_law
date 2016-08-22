extern crate tempdir;

use std::ffi::OsStr;
use std::sync::Arc;

use super::fixtures::read_scenario_fixture;
use super::settings::SceneSettings;

/// A command result is the outputs of a command (streams and status code)
/// within a struct which has convenience assertion functions about those outputs
pub struct CmdResult {
    //tmpd is used for convenience functions for asserts against fixtures
    pub settings: Arc<SceneSettings>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CmdResult {
    /// asserts that the command resulted in a success (zero) status code
    pub fn success(&self) -> Box<&CmdResult> {
        assert!(self.success);
        Box::new(self)
    }

    /// asserts that the command resulted in a failure (non-zero) status code
    pub fn failure(&self) -> Box<&CmdResult> {
        assert!(!self.success);
        Box::new(self)
    }

    /// asserts that the command resulted in empty (zero-length) stderr stream output
    /// generally, it's better to use stdout_only() instead,
    /// but you might find yourself using this function if
    /// 1. you can not know exactly what stdout will be
    /// or 2. you know that stdout will also be empty
    pub fn no_stderr(&self) -> Box<&CmdResult> {
        assert_eq!("", self.stderr);
        Box::new(self)
    }

    /// asserts that the command resulted in empty (zero-length) stderr stream output
    /// unless asserting there was neither stdout or stderr, stderr_only is usually a better choice
    /// generally, it's better to use stderr_only() instead,
    /// but you might find yourself using this function if
    /// 1. you can not know exactly what stderr will be
    /// or 2. you know that stderr will also be empty
    pub fn no_stdout(&self) -> Box<&CmdResult> {
        assert_eq!("", self.stdout);
        Box::new(self)
    }

    /// asserts that the command resulted in stdout stream output that equals the
    /// passed in value, when both are trimmed of trailing whitespace
    /// stdout_only is a better choice unless stderr may or will be non-empty
    pub fn stdout_is<T: AsRef<str>>(&self, msg: T) -> Box<&CmdResult> {
        assert_eq!(String::from(msg.as_ref()).trim_right(), self.stdout.trim_right());
        Box::new(self)
    }

    /// like stdout_is(...), but expects the contents of the file at the provided relative path
    pub fn stdout_is_fixture<T: AsRef<OsStr>>(&self, file_rel_path: T) -> Box<&CmdResult> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.stdout_is(contents)
    }

    /// asserts that the command resulted in stderr stream output that equals the
    /// passed in value, when both are trimmed of trailing whitespace
    /// stderr_only is a better choice unless stdout may or will be non-empty
    pub fn stderr_is<T: AsRef<str>>(&self, msg: T) -> Box<&CmdResult> {
        assert_eq!(String::from(msg.as_ref()).trim_right(), self.stderr.trim_right());
        Box::new(self)
    }

    /// like stderr_is(...), but expects the contents of the file at the provided relative path
    pub fn stderr_is_fixture<T: AsRef<OsStr>>(&self, file_rel_path: T) -> Box<&CmdResult> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.stderr_is(contents)
    }

    /// asserts that
    /// 1. the command resulted in stdout stream output that equals the
    /// passed in value, when both are trimmed of trailing whitespace
    /// and 2. the command resulted in empty (zero-length) stderr stream output
    pub fn stdout_only<T: AsRef<str>>(&self, msg: T) -> Box<&CmdResult> {
        self.no_stderr().stdout_is(msg)
    }

    /// like stdout_only(...), but expects the contents of the file at the provided relative path
    pub fn stdout_only_fixture<T: AsRef<OsStr>>(&self, file_rel_path: T) -> Box<&CmdResult> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.stdout_only(contents)
    }

    /// asserts that
    /// 1. the command resulted in stderr stream output that equals the
    /// passed in value, when both are trimmed of trailing whitespace
    /// and 2. the command resulted in empty (zero-length) stdout stream output
    pub fn stderr_only<T: AsRef<str>>(&self, msg: T) -> Box<&CmdResult> {
        self.no_stdout().stderr_is(msg)
    }

    /// like stderr_only(...), but expects the contents of the file at the provided relative path
    pub fn stderr_only_fixture<T: AsRef<OsStr>>(&self, file_rel_path: T) -> Box<&CmdResult> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.stderr_only(contents)
    }

    pub fn fails_silently(&self) -> Box<&CmdResult> {
        assert!(!self.success);
        assert_eq!("", self.stderr);
        Box::new(self)
    }
}
