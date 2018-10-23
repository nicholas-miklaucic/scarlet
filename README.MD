# Scarlet
## Colors and color spaces made simple, for Rustaceans.

[![Build
Status](https://travis-ci.org/nicholas-miklaucic/scarlet.svg?branch=master)](https://travis-ci.org/nicholas-miklaucic/scarlet)
[![dependency status](https://deps.rs/repo/github/nicholas-miklaucic/scarlet/status.svg)](https://deps.rs/repo/github/nicholas-miklaucic/scarlet)

Humans and computers don't view color the same way, and so color descriptions like RGB don't
effectively describe colors in the way that humans perceive them and vice versa. Image processing
and other disciplines require the ability to interconvert and work with colors in a way that
respects both how colors are displayed and how they are perceived by the human eye. Scarlet makes
this not only possible, but simple and convenient. With Scarlet, you can:
 * Interconvert between different color representations
 * Work *with* colors in one representation *using* the concepts of a different one: for example,
   you can use a model of color luminance that accurately models human vision without leaving RGB,
   or modify a color's hue in CIELAB
 * Mix and average colors accurately, without kludges or results that look wrong to humans
 * Create perceptually-uniform color scales, colormaps, and gradients suitable for publication-quality visuals that don't
   mislead viewers
 * Convert a color to grayscale accurately and precisely
 * Accurately determine how far apart colors are perceptually.
 * And more!
 
## Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
scarlet = "1.0.0"
```

and this to your crate root:

```rust
extern crate scarlet;
```

## Documentation
Consult the documentation at [docs.rs](https://docs.rs/scarlet/).
 
## Contributing Guidelines
Before making a pull request, please consult the [contributing guidelines](CONTRIBUTING.md).

The gist of it is:
 * Running `cargo test` should result in all tests passing. If tests themselves are wrong, change
   those in the pull request and explain the errors. Don't disable tests to make `cargo test` pass
   unless you have a really good reason!
 * If you make changes to the public-facing API, you should make sure that those changes are
   consistent with best practices and explain why you feel the API should change.
 * If you make performance improvements to code that already works, benchmark the change with test
   code that does things users would want to do and publish the results showing that your
   improvement actually improved something.
 * If you add new functionality, you should have test cases that thoroughly test that
   functionality.

## General Philosophy
To look at the general philosophy and API design of Scarlet, please look at `api.org`. This is more
intended for developers.
