// Copyright 2015, Yuheng Chen.
// Copyright 2023, Ethiraric.
// See the LICENSE file at the top-level directory of this distribution.

//! YAML 1.2 implementation in pure Rust.
//!
//! # Usage
//!
//! This crate is [on github](https://github.com/Ethiraric/yaml-rust2) and can be used by adding
//! `yaml-rust2` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! yaml-rust2 = "0.6.0"
//! ```
//!
//! # Examples
//! Parse a string into `Vec<Yaml>` and then serialize it as a YAML string.
//!
//! ```
//! use yaml_rust2::{YamlLoader, YamlEmitter};
//!
//! let docs = YamlLoader::load_from_str("[1, 2, 3]").unwrap();
//! let doc = &docs[0]; // select the first YAML document
//! assert_eq!(doc[0].as_i64().unwrap(), 1); // access elements by index
//!
//! let mut out_str = String::new();
//! let mut emitter = YamlEmitter::new(&mut out_str);
//! emitter.dump(doc).unwrap(); // dump the YAML object to a String
//!
//! ```

#![doc(html_root_url = "https://docs.rs/yaml-rust2/0.5.0")]
#![cfg_attr(feature = "cargo-clippy", warn(clippy::pedantic))]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::match_same_arms,
        clippy::should_implement_trait,
        clippy::missing_errors_doc,
        clippy::missing_panics_doc,
        clippy::redundant_else,
    )
)]

extern crate linked_hash_map;

pub(crate) mod char_traits;
#[macro_use]
pub(crate) mod debug;
pub mod emitter;
pub mod parser;
pub mod scanner;
pub mod yaml;

// reexport key APIs
pub use crate::emitter::{EmitError, YamlEmitter};
pub use crate::parser::Event;
pub use crate::scanner::ScanError;
pub use crate::yaml::{Yaml, YamlLoader};
