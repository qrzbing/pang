# Change Log

## 0.1.3 - 2025-08-29

### Features

- Add Language to specify a Grammar, start symbol and etc.

### Changed

- Mutator in `pang_libafl` now takes a Language object.

### Removed

- Generator and Parser are combined into Grammar.

### Beaking Changes

- Rewrite Symbol, Parser and Generator. Now both generate and parse method can be called in Language.