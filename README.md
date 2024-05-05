<h1 align="center">
    builder-derive
</h1>

builder-derive provides two main macros, the builder and consumer macro, this crate mainly exists as a support crate for [disarmv7](https://github.com/ivario123/disarmv7) but could be used outside of that. For most cases there exist better alternatives such as [`derive_builder`](https://docs.rs/derive_builder/latest/derive_builder/) but that one requires unwraps which, in a builder should not be needed as the type system can gaurantee safe construction.

## License

This repository is licensed under the [MIT](./LICENSE) license and any contributions shall be licensed under the same license unless explicitly stated otherwise.
