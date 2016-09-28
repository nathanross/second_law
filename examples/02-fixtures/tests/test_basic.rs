#[macro_use]
extern crate second_law;

fn new_ucmd() -> second_law::UCommand {
    new_scene!().ucmd()
}

#[test]
fn sum_adds_numbers() {
    new_ucmd().arg("1").arg("2").succeeds().stdout_only("3");
}

#[test]
fn sum_works_with_zero() {
    new_ucmd().arg("2").arg("0").succeeds().stdout_only("2");
}

#[test]
fn sum_no_args_returns_zero() {
    new_ucmd().succeeds().stdout_only("0");
}

#[test]
fn sum_error_non_numeric() {
    new_ucmd().arg("2").arg("three").fails().stderr_only("failure: could not parse argument 'three'");
}

