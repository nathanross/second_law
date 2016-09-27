# Second Law

> "A robot must obey the orders given it by human beings except where such orders would conflict with the First Law."

> Second law of Asimov's Three laws of robotics.

Second Law is what you should use for calling your binaries (instead of calling proccess::Command directly) in your binary integration tests, and for asserting over their output streams, exit code, and filesystem side-effects.

Second law provides a calling and assertion syntax that is intuitively understandable, and makes the actual content of your tests the focus by moving all the boilerplate necessary for a cross-platform consistent, debuggable binary test into the background. 

### Naturally readable tests

The test case messages of this integration test have been hidden. Can you summarize what the binary is required to do?

(This example is an actual Stainless test you can run in one of the [example packages](https://github.com/nathanross/second_law/tree/master/examples/01-simple), there is an equivalent [vanilla (non-stainless) test](https://github.com/nathanross/second_law/blob/master/examples/01-simple/tests/test_basic.rs) in that package as well)

```rust
#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#[macro_use]
extern crate second_law;

describe! <hidden> {
    before_each {
        let mut ucmd = new_scene!().ucmd();
    }

    it "should <hidden>" {
        ucmd.arg("2").arg("3").succeeds().stdout_only("5");
    }

    it "should <hidden>" {
        ucmd.arg("2").arg("0").succeeds().stdout_only("2");        
    }

    it "should <hidden>" {
        ucmd.succeeds().stdout_only("0");        
    }

    it "should <hidden>" {
        ucmd.arg("2").arg("three").fails().stderr_only("failure: could not parse argument 'three'");
    }
}
```

### Fixtures without boilerplate

* built-in support for fixtures (data files for your tests) in your tests.

### Subcommand support without boilerplate:

* First class support for subcommands

### consistency without boilerplate:

* By-default (but optional) clearing of the environment and use of temporary directories
* Provides an object-oriented path class to enable filesystem operations within this temporary directory.
* multiple commands can be run in this temporary directory.

### easy debugging without boilerplate:

* If your test case includes multiple, dynamic calls, assertion failures will display the exact command being tested that failed.

## FAQ

### Where/How would I use this?

Second Law can be used independently, or as a complement to test harnesses and frameworks like stainless.

### Design goals

Second Law focuses on supporting the following use case:

* Rust binaries
* non-GUI testing
* minimally interactive
