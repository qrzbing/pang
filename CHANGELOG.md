# Change Log

## 0.1.4

### Features

- Add `EncodeCallback` and `DecodeCallback` in `Expansion` to customize the encoding and decoding process.
- Add `exp_cb` to add Callbacks to Expansion.

### Changed

- Now `Generator` is no need to add `TreeFixer`.

### Removed

- Remove `TreeFixer`, now they are defined (to fix) directly in `Expansion` by `EncodeCallback`.
- Remove `GrammarOptions` and `opts`, now user can use `DecodeCallback` to customize the decoding process.
- Remove `exp_with_opts`, now user can use `exp_cb` to add Callbacks to Expansion.

## 0.1.3 - 2025-08-29

### Features

- Add Language to specify a Grammar, start symbol and etc.

### Changed

- Mutator in `pang_libafl` now takes a Language object.

### Removed

- Generator and Parser are combined into Grammar.

### Beaking Changes

- Rewrite Symbol, Parser and Generator. Now both generate and parse method can be called in Language.