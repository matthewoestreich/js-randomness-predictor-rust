<h1 align="center">jsrp</h1>
<p align="center">
<small>A <b><u>J</u></b>ava<b><u>S</u></b>cript <b><u>R</u></b>andomness <b><u>P</u></b>redictor</small>

<p align="center">
  <a href="https://crates.io/crates/jsrp">
    <img src="https://img.shields.io/crates/v/jsrp" alt="crates.io" />
  </a>
</p>

<p align="center">Predict JS Math.random output in Node, Chrome, and Firefox with Rust!<br/>You can use this package programmatically, or via CLI.</p>

---

# Installation

**To use programmatically**

```bash
cargo add jsrp
```

**To use CLI**

This installs `jsrp` globally! You will be able to use the `jsrp` command system wide.

```bash
cargo install jsrp
```

# Usage

**Firefox**

```rust
use jsrp::FirefoxPredictor;
let mut ffp = FirefoxPredictor::new(vec![/*
 4 random numbers copied from
 using Math.random in Firefox.
*/]);

let next = ffp.predict_next();
// Run another Math.random() in
// Firefox to validate `next`.
```

**Chrome**

```rust
use jsrp::ChromePredictor;
let mut chrp = ChromePredictor::new(vec![/*
 4 random numbers copied from
 using Math.random in Chrome.
*/]);

let next = chrp.predict_next();
// Run another Math.random() in
// Chrome to validate `next`.
```

**Node**

You **must** provide a `Node.js` version (just the `major` version) when calling `new`! 

You can get your current `Node.js` version by running the following command in your terminal:

```bash
node -p "process.versions.node.split('.')[0]"
# 24
```

Once you have your `Node.js` `major` version: (which we are using `24` as an example):

```rust
use jsrp::NodePredictor;
let mut n_v24_p = NodePredictor::new(
    24, // <- Node.js major version 
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = n_v24_p.predict_next();
// Run another Math.random() in
// Node.js to validate `next`.
```

**Make Predictions for Different Node.js Versions**

If you have a sequence of random numbers that someone generated in `Node.js v22.x.x` (or whatever version) you can still run the predictor against them, regardless of your current `Node.js` version.

Just specify "that" version:

```rust
use jsrp::NodePredictor;
let mut n_vX_p = NodePredictor::new(
    some_nodejs_major_version,
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = n_vX_p.predict_next();
// Run another Math.random() in
// Node.js to validate `next`.
```

**Safari**

```rust
use jsrp::SafariPredictor;
let mut sp = SafariPredictor::new(vec![/*
 4 random numbers copied from
 using Math.random in Safari.
*/]);

let next = sp.predict_next();
// Run another Math.random() in
// Safari to validate `next`.
```

# CLI

Use `jsrp --help` to get a full list of commands/arguments (as well as their shorthand equivalent).

Each number within `--sequence` should be separated by a space.

By default we provide 10 predictions (if `--predictions` was not provided).

```bash
# Node - make 12 predictions
# Must provide a Node.js major version!
# Major versions must start with a "v"
jsrp node --sequence 0.1 0.2 0.3 --major-version v24 --predictions 12
jsrp node --sequence 0.1 0.2 0.3 --major-version v24
# Shorthand
jsrp node -s 0.1 0.2 0.3 -m v24 -p 12

# Firefox
jsrp firefox -s ... -p N
# Chrome
jsrp chrome -s ... -p N
# Safari
jsrp safari -s ... -p N
```
