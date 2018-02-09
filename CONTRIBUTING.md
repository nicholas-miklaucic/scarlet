# Contributing
Any pull request should follow all of these rules unless you have a really good reason not to and
you explain it.

Note that the current state of the library is a little bit hypocritical: work is underway to fix
this.

## Rule 1: Tests
Scarlet uses the standard Cargo testing suite, and any pull requests should also work with
this. That means:
 * If you add new functionality, you should add tests for every part of that functionality.
 * If you change existing functionality, you should change any tests that break so that they pass.
 * If you make changes that don't modify functionality, you don't need to change tests unless any
   break, in which case you should fix your code or the test, whichever is wrong.

The golden rule is that **all tests should pass.** Scarlet uses [Travis CI](https://travis-ci.org)
for continuous integration, so this shouldn't be that onerous.

## Rule 2: Documentation
Any public-facing new code should be well-documented. What that means, exactly, is covered in the
next rule.

Any source code that does unusual or surprising things should have source comments that explain
it. Any performance-readability tradeoffs should have associated measurements to demonstrate the
performance improvement, either in comments or in the pull request itself.

## Rule 3: API
As a general rule, any change to the public API should comply with [the Rust API
guidelines](https://rust-lang-nursery.github.io/api-guidelines/). There are often good reasons for
not following one of these: if that applies to you, you should explicitly state so in your PR,
including the guideline you're breaking (ideally the official name they give it, such as
`C-SMART-PTR`, and the reason you're breaking it.

