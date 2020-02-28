# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.2.0] - 202-02-28
### Changed
- Rely on `tide` version 0.6.0 (up from 0.5.1).
- Rearrange example directory a bit. Test `.html` files are now in their own subdirectory.
### Removed
- The library itself no longer depends on unstable `tide` features, only the example programs have such a dependency. Thanks to @eribol bringing up this problem in issue #4.