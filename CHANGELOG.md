# klvm_tools_rs Changelog

## 0.1.34
### Fixed
- Fixed chiklisp compilation issues


## 0.1.33
### Changed
- Set macOS deployment target to 10.14
- Ensure we flush streams in case the runtime system doesn't get a chance
### Fixed
- Fix erroneous detection of recursion when two similar inline siblings


## 0.1.32
Skipped

## 0.1.31 Chik KLVM Tools Rust 2023-04-17

### Added

- defconst was added.
- hierarchikl debug was added.
- klvm command linetools: supported more command line features in both compiler front-ends.

## 0.1.35 

- embed-file was added.
- &rest arguments.
- new bls and sec256 operators.

## 0.1.36

- modern lambda added
- updated some internal data strucutres and call interfaces to support env variable renaming at during closure generation / lambda capture, or any step during transformation.

## 0.1.37

- First npm publish with a Program-like object reminiscent of
  chik.types.blockchain_format.program.Program

## 0.1.38

- Uncurry fix, typescript type improvements for npm personality.