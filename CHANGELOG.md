# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1](https://github.com/wcampbell0x2a/librarium/compare/v0.6.0...v0.6.1) - 2026-09-02

### Other

- Add release-plz yml
- Merge pull request #82 from wcampbell0x2a/update-all-the-things
- Update crates to latest compatible versions
- Add support for concatenated archives, such as a Linux kernel initramfs:
  `ArchiveReader::end_offset`, `next_segment_offset`, and `segment_format`
- Fix `ArchiveReader::from_reader_with_offset` reading from the current position
  of the reader when the offset is `0`

## [0.6.0] - 06-07-2025
- Move MutWriter to dedicated module
- Add documentation
- Add `TryFrom<&std::fs::Metadata>` for Header
- Add readme and library support matrix
- Add NewC with CRC headers (b"070702")
- Fix name field not containing null terminator

## [0.5.0] - 12-22-2025
- Update deku to v0.20.2
- Add multi-threaded file retrieval
- Add progress and status bar to `dl`
- Add no_std support with `std` features

## [0.4.0] - 05-03-2025
- Update deku to v0.19.0 (#58)

## [0.3.1] - 11-09-2024
- Fix Cargo.toml keywords

## [0.3.0] - 11-09-2024
- Add support for Reading and Writing both Odc and Newc CPIO archives
- Cleanup up all documentation, adding examples and checking readme


## [0.2.0] - 09-26-2024
- First official release
