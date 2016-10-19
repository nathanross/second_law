#[macro_use]
extern crate second_law;

fn new_ucmd() -> second_law::UCommand {
    new_scene!().ucmd()
}

#[test]
fn average_ignores_newlines() {
    new_ucmd().arg("newlines.txt").succeeds().stdout_only("20");
}

#[test]
fn average_single_numbers() {
    new_ucmd().arg("one_number.txt").succeeds().stdout_only("10");
}

#[test]
fn average_many_numbers() {
    new_ucmd().arg("many_numbers.txt").succeeds().stdout_only("20");
}

#[test]
fn average_empty_file() {
    new_ucmd().arg("empty.text").fails().stderr_only("failure: this file has no number lines'");
}

