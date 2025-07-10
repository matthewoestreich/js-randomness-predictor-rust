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

- You must use the matching predictor for the environment! **Meaning, if random numbers were generatedin Firefox, you must use the Firefox predictor, etc..**
- You must generate the initial sequence, as well as expected results, **from within the same context**. [Please see here for a more detailed explanation](#generation-context)
- We recommend you provide at least 4 random numbers in the initial sequence
- [Please see here for a list of known issues!](#known-issues)

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

- You **must** provide a `Node.js` version (just the `major` version) when calling `new`! 

```bash
# You can get your current `Node.js` version 
# by running the following command in your 
# terminal:
node -p "process.versions.node.split('.')[0]"
#-> 24
```

Once you have your `Node.js` `major` version: (which we are using `24` as an example):

```rust
use jsrp::{NodePredictor, NodeJsMajorVersion};
let mut np_v24 = NodePredictor::new(
    NodeJsMajorVersion::V24,
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = np_v24.predict_next()?;
// Run another Math.random() in
// Node.js to validate `next`.
```

**Make Predictions for Different Node.js Versions**

If you have a sequence of random numbers that someone generated in `Node.js v22.x.x` (or whatever version) you can still run the predictor against them, regardless of your current `Node.js` version.

Just specify "that" version:

```rust
use jsrp::NodePredictor;
let mut np_vX = NodePredictor::new(
    that_nodejs_major_version,
    vec![/*
    4 random numbers copied from
    using Math.random in the specific
    Node.js version you provided.
    */],
);

let next = np_vX.predict_next()?;
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
- Use `jsrp <environment> --help` to get a full list of commands/arguments for a specific environment.
- Each number within `--sequence` should be separated by a space.
- By default we provide 10 predictions (if `--predictions` was not provided).
- You can export results to JSON (more info below).
- You can provide expected results so that we can automatically validate our predictions (more info below).

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

**Validate Expected Results**

If you already have the expected sequence, you can provide it via the `--expected`, or `-x`, flag.  If provided, we will automatically validate our predictions.

```bash
# This actually works! Try it in the CLI
jsrp node\
  --major-version v24\
  --sequence 0.15825075235897956\
     0.6837830031246955\
     0.2352848927050296\
     0.6995244175841968\
  --expected 0.32903013894382993

# {
#   "environment": "Node.js v24",
#   "expected": [
#     0.32903013894382993
#   ],
#   "is_accurate": true,
#   "predictions": [
#     0.32903013894382993
#   ],
#   "sequence": [
#     0.15825075235897956,
#     0.6837830031246955,
#     0.2352848927050296,
#     0.6995244175841968
#   ]
# }
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

### Generation Context

- You can't generate the initial sequence from the console in "browser tab A", and then generate the expected results from the console in a different browser tab. **Both the sequence and expected numbers should have been generated in "browser tab A"**
- If you do `node -p "Array.from({ length: 4 }, Math.random)"` to generate the initial sequence, **you will have no way of verifying our predictions**. Instead, **you would need to enter the Node REPL *(because all generated random numbers would be from the same context)***
  - eg enter `$ node` from terminal, and then once in REPL `> Array.from({ length: 4 }, Math.random)` for initial sequence and `> Array.from({ length: 10 }, Math.random)` for expected results.

## Node

### Random Number Pool Exhaustion

TLDR; If `number of predictions` + `sequence length` > `64`, we cannot make accurate predictions. We call this "pool exhaustion".

**Why does this happen?**

- Node generate 64 "random" numbers at a time, which they cache in a "pool"
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.cc;l=17-27) that shows how they build the cache
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.h;l=24;drc=75a8035abe03764596f30424030465636e82aa70;bpv=0) showing cache size
- A seed is used to generate these "random" numbers
- Solving for that seed is what allows us to predict future `Math.random` output
- When you call `Math.random()` they grab a number from this "pool" and return it to you
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/builtins/math.tq;l=515-530) that shows how they pull from cache and return a random number to you, [specifically here](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/builtins/math.tq;l=529)
- When that "pool" is exhausted, they generate a new "pool", **with a new/different seed**
  - [Source code](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.cc;l=35-71) that shows how they refill the cache, [specifically here](https://source.chromium.org/chromium/chromium/src/+/main:v8/src/numbers/math-random.cc;l=61-65)
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
