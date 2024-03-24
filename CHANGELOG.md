# Changelog

## Upcoming

**Breaking Changes**:

- The `encoding` library has been replaced with `encoding_rs`. If you use the
`trap` of `YamlDecoder`, this change will make your code not compile.
An additional enum `YamlDecoderTrap` has been added to abstract the
underlying library and avoid breaking changes in the future. This
additionally lifts the `encoding` dependency on _your_ project if you were
using that feature.
  - The signature of the function for `YamlDecoderTrap::Call` has changed:
  - The `encoding::types::DecoderTrap` has been replaced with `YamlDecoderTrap`.
    ```rust
    // Before, with `encoding::types::DecoderTrap::Call`
    fn(_: &mut encoding::RawDecoder, _: &[u8], _: &mut encoding::StringWriter) -> bool;
    // Now, with `YamlDecoderTrap::Call`
    fn(_: u8, _: u8, _: &[u8], _: &mut String) -> ControlFlow<Cow<'static str>>;
    ```
    Please refer to the `YamlDecoderTrapFn` documentation for more details.

**Features**:

- Tags can now be retained across documents by calling `keep_tags(true)` on a
`Parser` before loading documents.
([#10](https://github.com/Ethiraric/yaml-rust2/issues/10)
([#12](https://github.com/Ethiraric/yaml-rust2/pull/12))

- `YamlLoader` structs now have a `documents()` method that returns the parsed
documents associated with a loader.

- `Parser::new_from_str(&str)` and `YamlLoader::load_from_parser(&Parser)` were added.

**Development**:

## v0.7.0

**Features**:

- Multi-line strings are now
[emitted using block scalars](https://github.com/chyh1990/yaml-rust/pull/136).

- Error messages now contain a byte offset to aid debugging.
([#176](https://github.com/chyh1990/yaml-rust/pull/176))

- Yaml now has `or` and `borrowed_or` methods.
([#179](https://github.com/chyh1990/yaml-rust/pull/179))

- `Yaml::load_from_bytes()` is now available.
([#156](https://github.com/chyh1990/yaml-rust/pull/156))

- The parser and scanner now return Err() instead of calling panic.

**Development**:

- The documentation was updated to include a security note mentioning that
yaml-rust is safe because it does not interpret types.
([#195](https://github.com/chyh1990/yaml-rust/pull/195))

- Updated to quickcheck 1.0.
([#188](https://github.com/chyh1990/yaml-rust/pull/188))

- `hashlink` is [now used](https://github.com/chyh1990/yaml-rust/pull/157)
instead of `linked_hash_map`.

## v0.6.0

**Development**:

- `is_xxx` functions were moved into the private `char_traits` module.

- Benchmarking tools were added.

- Performance was improved.

## v0.5.0

- The parser now supports tag directives.
([#35](https://github.com/chyh1990/yaml-rust/issues/35)

- The `info` field has been exposed via a new `Yaml::info()` API method.
([#190](https://github.com/chyh1990/yaml-rust/pull/190))
