#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#[macro_use]
extern crate second_law;

describe! sum {
    before_each {
        let mut ucmd = new_scene!().ucmd();
    }

    it "should add numbers" {
        ucmd.arg("2").arg("3").succeeds().stdout_only("5");
    }

    it "should not have an error with 0" {
        ucmd.arg("2").arg("0").succeeds().stdout_only("2");        
    }

    it "should default to returning 0" {
        ucmd.succeeds().stdout_only("0");        
    }

    it "should fail on non-numeric input" {
        ucmd.arg("2").arg("three").fails().stderr_only("failure: could not parse argument 'three'");
    }
}
