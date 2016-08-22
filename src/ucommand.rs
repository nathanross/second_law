use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::process::{Command, Stdio, Child};
use std::str::from_utf8;
use std::sync::Arc;
use std::path::PathBuf;

use super::cmdresult::CmdResult;
use super::fixtures::read_scenario_fixture;
use super::common::log_info;
use super::settings::SceneSettings;

static ALREADY_RUN: &'static str = "you have already run this UCommand, if you want to run \
                                    another command in the same test, use scene.ucmd()";
static MULTIPLE_STDIN_MEANINGLESS: &'static str = "Ucommand is designed around a typical use case of: provide args and input stream -> spawn process -> block until completion -> return output streams. For verifying that a particular section of the input stream is what causes a particular behavior, use the Command type directly.";

/// A UCommand is a wrapper around an individual Command that provides several additional features
/// 1. it has convenience functions that are more ergonomic to use for piping in stdin, spawning the command
///       and asserting on the results.
/// 2. it tracks arguments provided so that in test cases which may provide variations of an arg in loops
///     the test failure can display the exact call which preceded an assertion failure.
/// 3. it provides convenience construction arguments to set the Command working directory and/or clear its environment.
pub struct UCommand {
    pub raw: Command,
    comm_string: String,
    settings: Arc<SceneSettings>,
    has_run: bool,
    stdin: Option<Vec<u8>>
}

impl UCommand {
    pub fn new<T: AsRef<OsStr>, U: AsRef<OsStr>>(
        invoked: T,
        settings: Arc<SceneSettings>,
        env_clear: bool,
        curdir: Option<U>
    ) -> UCommand {
        let curdir_used = if let Some(curdir_val) = curdir {
            PathBuf::from(&curdir_val.as_ref())
        } else {
            PathBuf::from(settings.as_ref().tmpd.path())
        };
//        let  = String::from(&(*tmpd.as_ref().path().to_str().unwrap()));
        UCommand {
            settings: settings,
            has_run: false,
            raw: {
                let mut cmd = Command::new(invoked.as_ref());
                cmd.current_dir(&curdir_used);
                if env_clear {
                    if cfg!(windows) {
                        // %SYSTEMROOT% is required on Windows to initialize crypto provider
                        // ... and crypto provider is required for std::rand
                        // From procmon: RegQueryValue HKLM\SOFTWARE\Microsoft\Cryptography\Defaults\Provider\Microsoft Strong Cryptographic Provider\Image Path
                        // SUCCESS  Type: REG_SZ, Length: 66, Data: %SystemRoot%\system32\rsaenh.dll"
                        for (key, _) in env::vars_os() {
                            if key.as_os_str() != "SYSTEMROOT" {
                                cmd.env_remove(key);
                            }
                        }
                    } else {
                        cmd.env_clear();
                    }
                }
                cmd
            },
            comm_string: String::from(invoked.as_ref().to_str().unwrap()),
            stdin: None
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> Box<&mut UCommand> {
        if self.has_run {
            panic!(ALREADY_RUN);
        }
        self.comm_string.push_str(" ");
        self.comm_string.push_str(arg.as_ref().to_str().unwrap());
        self.raw.arg(arg.as_ref());
        Box::new(self)
    }

    /// like arg(...), but uses the contents of the file at the provided relative path as the argument
    pub fn arg_fixture<S: AsRef<OsStr>>(&mut self, file_rel_path: S) -> Box<&mut UCommand> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.arg(contents)
    }

    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> Box<&mut UCommand> {
        if self.has_run {
            panic!(MULTIPLE_STDIN_MEANINGLESS);
        }
        for s in args {
            self.comm_string.push_str(" ");
            self.comm_string.push_str(s.as_ref().to_str().unwrap());
        }

        self.raw.args(args.as_ref());
        Box::new(self)
    }

    /// provides stdinput to feed in to the command when spawned
    pub fn pipe_in<T: Into<Vec<u8>>>(&mut self, input: T) -> Box<&mut UCommand> {
        if self.stdin.is_some() {
            panic!(MULTIPLE_STDIN_MEANINGLESS);
        }
        self.stdin = Some(input.into());
        Box::new(self)
    }

    /// like pipe_in(...), but uses the contents of the file at the provided relative path as the piped in data
    pub fn pipe_in_fixture<S: AsRef<OsStr>>(&mut self, file_rel_path: S) -> Box<&mut UCommand> {
        let contents = read_scenario_fixture(&self.settings, file_rel_path);
        self.pipe_in(contents)
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> Box<&mut UCommand> where K: AsRef<OsStr>, V: AsRef<OsStr> {
        if self.has_run {
            panic!(ALREADY_RUN);
        }
        self.raw.env(key, val);
        Box::new(self)
    }

    /// Spawns the command, feeds the stdin if any, and returns the
    /// child process immediately.
    pub fn run_no_wait(&mut self) -> Child {
        if self.has_run {
            panic!(ALREADY_RUN);
        }
        self.has_run = true;
        log_info("run", &self.comm_string);
        let mut result = self.raw
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("There was an error running the provided command. Run cargo test with --verbose to see which command caused the failure");

        if let Some(ref input) = self.stdin {
            result.stdin
                .take()
                .unwrap_or_else(
                    || panic!(
                        "Could not take child process stdin"))
                .write_all(&input)
                .unwrap_or_else(|e| panic!("{}", e));
        }

        result
    }

    /// Spawns the command, feeds the stdin if any, waits for the result
    /// and returns a command result.
    /// It is recommended that you instead use succeeds() or fails()
    pub fn run(&mut self) -> CmdResult {
        let prog = self.run_no_wait().wait_with_output().unwrap();

        CmdResult {
            settings: self.settings.clone(),
            success: prog.status.success(),
            stdout: from_utf8(&prog.stdout).unwrap().to_string(),
            stderr: from_utf8(&prog.stderr).unwrap().to_string(),
        }
    }

    /// Spawns the command, feeding the passed in stdin, waits for the result
    /// and returns a command result.
    /// It is recommended that, instead of this, you use a combination of pipe_in()
    /// with succeeds() or fails()
    pub fn run_piped_stdin<T: Into<Vec<u8>>>(&mut self, input: T) -> CmdResult {
        self.pipe_in(input).run()
    }

    /// Spawns the command, feeds the stdin if any, waits for the result,
    /// asserts success, and returns a command result.
    pub fn succeeds(&mut self) -> CmdResult {
        let cmd_result = self.run();
        cmd_result.success();
        cmd_result
    }

    /// Spawns the command, feeds the stdin if any, waits for the result,
    /// asserts success, and returns a command result.
    pub fn fails(&mut self) -> CmdResult {
        let cmd_result = self.run();
        cmd_result.failure();
        cmd_result
    }
}
