# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2023-08-23

### Breaking changes

- `new` constructor got renamed to `with_default`
- `new_with_map` constructor got renamed to `from_map_with_default`, and its
  arguments were switched around
- The new minimum supported Rust version is 1.71.0

### Added

- A lot more methods are now forwarded
- `with_fn` and `from_map_with_fn` constructors were added to allow creating a
  default from a closure.
- A `get_default` method to get an owned version of the default.

### Changed

- Values now don't have to implement `Clone`. They only need to implement
  `Default`. This is especially useful for values that are an `Option<T>` of
  where `T` does not implement `Clone`.
