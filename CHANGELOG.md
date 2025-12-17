# Change Log

## 0.2.3 - 2025-12-17

### Beaking Changes

- All `parse` methods in Lang/Grammar/... are deprecated, and will be removed in 0.3.0.

### Features

- Add `Switch` Non-Terminal, which is used when a struct depends on previous type, like Ikev2 protocol.
- Add pang_derive, which supports three derives: `PangLabel`, `ToTree` and `ToGrammar`. Now user is no need to write Grammar on their own.
    - `PangLabel` is used to get the name of a Struct.
    - `ToTree` can convert a struct with value to Derivation Tree.
    - `ToGrammar` generate Grammmar for a Struct.

## 0.2.2 - 2025-10-16

### Beaking Changes

- Remove `generate` trait from `TerminalKind` and `NonTerminalKind`.
- Remove `as_has_length` trait from `TerminalKind`.

### Changed

- Rename `nomt` to `nt_nom`.

## 0.2.1 - 2025-09-28

### Features

- Add `NOrMore` Non-Terminal, user can use `nomt` to create it.

### Changed

- Rewrite `Symbol::NonTerminal`, now users can define custom non-terminal through `NonTerminalKind` by themself.

## 0.2.0 - 2025-09-20

### Features

- `pang_derive`, simple tools for pre-generate grammar.
    - In 0.2.x, `pang_derive` will add kaitai support.

## 0.1.5 - 2025-09-11

### Features

- `Grammar` can be described more clearly.
- Add `label` and some helper functions to `Terminal`.

### Beaking Changes

- Now `Language` accepts `&Grammar`.

## 0.1.4 - 2025-09-04

### Features

- Add `EncodeCallback` and `DecodeCallback` in `Expansion` to customize the encoding and decoding process.
    - Add `exp_dc` to add DecodeCallback to Expansion.
    - Add `exp_ec` to add EncodeCallback to Expansion.
    - Add `exp_cb` to add DecodeCallback and EncodeCallback to Expansion.
- Add `fix` method for `DerivationTree`, which fix the derivation tree by applying `EncodeCallback` in `Expansion`.

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