# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.2.0] - 2023-04-17

### Added

- Implement `Pinboard::get_ref` to safely borrow the data in the pinboard
  - In particular this works for non-`Clone` types

## [2.1.0] - 2020-11-01

### Added

- Implement `Pinboard::new_empty`

## Fixed

- Upgrade `crossbeam` to `0.8.0`
- Upgrade `crossbeam-epoch` to `0.9.0`

## [2.0.1] - 2019-01-03

### Fixed

- Upgrade `crossbeam` to `0.6.0`
- Upgrade `crossbeam-epoch` to `0.7.0`

## [2.0.0] - 2017-12-17

### Changed

- Switch to `crossbeam-epoch` as GC manager
  - Ensures that `T` instances are dropped when they're removed from the pinboard

## [1.4.1] - 2017-12-04

### Fixed

- Fix atomic ordering on `Pinboard::set` and `Pinboard::clear`

## [1.4.0] - 2017-09-08

### Fixed

- Upgrade `crossbeam` to `0.3.0`

## [1.3.0] - 2017-08-15

### Added

- Implement `From<Option<T>>` for `Pinboard<T>`

## [1.2.0] - 2017-06-14

### Added

- `Pinboard<T>` implements `Display` and `Debug` wherever `T` does

## [1.1.0] - 2017-06-09

### Added

- New wrapping `NonEmptyPinboard` that guarantees the pinboard is always populated

## [1.0.0] - 2017-06-06

### Added

- Initial release
