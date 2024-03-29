# Pinboard

[![Crates.io - Pinboard](https://img.shields.io/crates/v/pinboard.svg)](https://crates.io/crates/pinboard) [![Build Status](https://github.com/bossmc/pinboard/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/bossmc/pinboard/actions/workflows/rust.yml?query=branch%3Amaster) [![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/873) [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

An eventually-consistent, lock-free, mutable store of shared data.

> Just stick it on the pinboard!

## Documentation

https://docs.rs/pinboard/

## Usage

Install from crates.io by adding `pinboard` to your `Cargo.toml`:

```text
[dependencies]
pinboard = "2.0.0"
```

Now you can create a Pinboard, share it between your users (be they `Futures`, threads or really anything else) and start sharing data!

```rust,no_run
use pinboard::NonEmptyPinboard;
use std::{thread, time::Duration};

let weather_report = NonEmptyPinboard::new("Sunny");

crossbeam::scope(|scope| {
  scope.spawn(|_| {
    thread::sleep(Duration::from_secs(10));
    weather_report.set("Raining");
  });
  scope.spawn(|_| {
    loop {
      println!("The weather is {}", weather_report.get_ref());
      thread::sleep(Duration::from_secs(1));
    }
  });
});
```
