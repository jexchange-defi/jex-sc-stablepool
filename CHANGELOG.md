# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.0]

### Changed

- upgrade to `sdk-rs 0.53.2`

## [1.5.0]

### Added

- add `estimateAmountIn` view

## [1.4.0]

### Added

- access control: enable swap for allowed smart contracts only

## [1.3.1]

### Fixed

- fix stableswap AMM with underlying prices

## [1.3.0]

### Added

- swap event

## [1.2.0]

### Added

- underlying token price source management for yield bearer tokens (eg: SEGLD-EGLD)

### Changed

- upgrade to `sdk-rs 0.47.8`

## [1.1.0]

### Added

- add view on all storage mappers

### Changed

- update to `sdk-rs 0.45.2`

### Fixed

- check number of tokens during deployment
- check amplification factor during deployment
- use constant to save redundant calculus
- check minimum amount out > 0
- check all tokens are unique during deployment

## [1.0.0]

### Added

Initial version
