<h1 align="center">jsrp</h1>
<p align="center">
<small>A <b><u>J</u></b>ava<b><u>S</u></b>cript <b><u>R</u></b>andomness <b><u>P</u></b>redictor</small>

<p align="center">
  <a href="https://crates.io/crates/jsrp">
    <img src="https://img.shields.io/crates/v/jsrp" alt="crates.io" />
  </a>
</p>

<p align="center">Predict JS Math.random output in Node, Chrome, Firefox, and Safari with Rust!<br/>You can use this package programmatically, or via CLI.</p>

---

# Important

- [Please see here for a list of known issues!](#known-issues)
- We recommend you provide at least 4 random numbers in the initial sequence
- You must use the matching predictor for the environment! **Meaning, if random numbers were generatedin Firefox, you must use the Firefox predictor, etc..**

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

[See here for known Firefox issues](#firefox)

```rust
use jsrp::FirefoxPredictor;
let mut ffp = FirefoxPredictor::new(vec![/*
 4 random numbers copied from
 using Math.random in Firefox.
*/]);

let next = ffp.predict_next()?;
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

let next = chrp.predict_next()?;
// Run another Math.random() in
// Chrome to validate `next`.
```

**Node**

[See here for known Node issues](#node)

You **must** provide a `Node.js` version (just the `major` version) when calling `new`! 

You can get your current `Node.js` version by running the following command in your terminal:

```bash
node -p "process.versions.node.split('.')[0]"
# 24
```

Once you have your `Node.js` `major` version: (which we are using `24` as an example):

```rust
use jsrp::{NodePredictor, NodeJsMajorVersion};
let mut n_v24_p = NodePredictor::new(
    NodeJsMajorVersion::V24,
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = n_v24_p.predict_next()?;
// Run another Math.random() in
// Node.js to validate `next`.
```

**Make Predictions for Different Node.js Versions**

If you have a sequence of random numbers that someone generated in `Node.js v22.x.x` (or whatever version) you can still run the predictor against them, regardless of your current `Node.js` version.

Just specify "that" version:

```rust
use jsrp::NodePredictor;
let mut n_vX_p = NodePredictor::new(
    that_nodejs_major_version,
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = n_vX_p.predict_next()?;
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

let next = sp.predict_next()?;
// Run another Math.random() in
// Safari to validate `next`.
```

# CLI

- Use `jsrp --help` to get a full list of commands/arguments (as well as their shorthand equivalent).
- Each number within `--sequence` should be separated by a space.
- By default we provide 10 predictions (if `--predictions` was not provided).

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

**Export Results to JSON**

```bash
# You can add `--export` (`-e` for short) to any command,
# which will export the results to .json.
# The file path MUST end in .json!!!
jsrp <environment> -s ... -e ./some/path/results.json
```

---

# Known Issues

## Node

### Random Number Pool Exhaustion

TLDR; If `number of predictions` + `sequence length` > `64`, we cannot make accurate predictions. We call this "pool exhaustion".

**Why does this happen?**

- Node generate 64 "random" numbers at a time, which they cache in a "pool"
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.cc;l=19-27) that shows how they build the cache
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.h;l=24;drc=75a8035abe03764596f30424030465636e82aa70;bpv=0) showing cache size
- A seed is used to generate these "random" numbers
- Solving for that seed is what allows us to predict future `Math.random` output
- When you call `Math.random()` they grab a number from this "pool" and return it to you
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/builtins/math.tq;l=515-530) that shows how they pull from cache and return a random number to you, [specifically here](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/builtins/math.tq;l=529)
- When that "pool" is exhausted, they generate a new "pool", **with a new/different seed**
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.cc;l=35-56) that shows how they refill the cache
- This means we cannot make accurate predictions for the new pool using the old pools seed

**How we handle it**

- When using the CLI, if `number of predictions` + `sequence length` > `64`, we will show a warning as well as truncate "number of predictions" to be within the allowed bounds.
- For example, if you provided `[1, 2, 3, 4]` as the sequence, which has a length of 4, the max amount of predictions we can successfully make is 60 (because 64 - 4 = 60)
- If the "length of the sequence" **on it's own** is >= 64, we will throw an error because we have no room for predictions, the entire pool was exhausted on the sequence

---

## Firefox

### Random Number Generation in Console

You must disable "Instant Evaluation", otherwise your predictions may show incorrectly. Especially if you use more than one call to generate the initial sequence + expected values.

**How to disable**

<img width="1920" alt="Firefox_DisableConsoleInstantEvaluation" src="/.github/Firefox_DisableConsoleInstantEvaluation.png" />

**If you do not want to disable "Instant Evaluation"**

- You'll need to generate initial sequence + expected values in one command.
- So instead of using two (or more) calls to `Math.random`:

```js
/** Pretend this is the console */
// Output used as initial sequence.
Array.from({ length: 4 }, Math.random);
// Output used for validating predictions.
Array.from({ length: 10 }, Math.random);
```

- You'll need to do:

```js
/** Pretend this is the console */
// Only use one call! Manually separate numbers!
Array.from({ length: 6 }, Math.random);
[
  // --------------------|
  0.5654163987207667, // |
  0.7409356182179403, // | --> Use "these" numbers as initial sequence
  0.46136469064448193, //|
  0.18124646315195891, //|
  // --------------------|
  0.25678544986069995, // --> Use the rest of the numbers for validation
  0.5543550504255771,
];
```

# Safari

NONE

# Chrome

NONE
