# Second Law

 "A robot must obey the orders given it by human beings except where such orders would conflict with the First Law."

 Second law of Asimov's Three laws of robotics.

Second Law is a swiss army knife for writing naturally-reading integration tests for your rust binaries. Second Law handles the boilerplate that would otherwise crowd a consistent, easily-debugged binary test, and provides a calling and assertion syntax that is intuitively understandable.

### Where/How would I use this?

Second Law is what you should use for calling your binaries (instead of calling proccess::Command directly) in your binary integration tests and asserting over their output streams, exit code, and filesystem side-effects.

Second Law can be used independently, or as a complement to test harnesses and frameworks like stainless.

### Features

**naturally reading**

* piping in stdin is as easy as ```.pipe_in("hello world")``` - it accepts any value that can be converted into a byte array.
* no ambiguity about what output stream is being asserted over. The stream the value is being tested against is right there in the name

**Fixtures without boilerplate**

* built-in support for fixtures (data files for your tests) in your tests.

**subcommand support without boilerplate:**

* First class support for subcommands

**consistency without boilerplate:**

* By-default (but optional) clearing of the environment and use of temporary directories
* Provides an object-oriented path class to enable filesystem operations within this temporary directory.
* multiple commands can be run in this temporary directory.

**easy debugging without boilerplate:**

* If your test case includes multiple, dynamic calls, assertion failures will display the exact command being tested that failed.

### Design goals

Second Law focuses on supporting the following use case:

* Rust binaries
* non-GUI testing
* minimally interactive
