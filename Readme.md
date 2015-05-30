# yaml-rust

The missing YAML 1.2 implementation for Rust.

[![Build Status](https://travis-ci.org/chyh1990/yaml-rust.svg?branch=master)](https://travis-ci.org/chyh1990/yaml-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/scf47535ckp4ylg4?svg=true)](https://ci.appveyor.com/project/chyh1990/yaml-rust)

`yaml-rust` is a pure Rust YAML 1.2 implementation without
any FFI and crate dependencies, which enjoys the memory safe 
property and other benefits from the Rust language. 
The parser is havily influenced by `libyaml` and `yaml-cpp`.

NOTE: This library is still under heavily development.

## Quick Start

Adding the following to the Cargo.toml in your project:

```
[dependencies.yaml-rust]
git = "https://github.com/chyh1990/yaml-rust.git"
```

and import using *extern crate*:

```.rust
extern crate yaml_rust;
```

Use `yaml::YamlLoader` to load the YAML documents and access it
as Vec/HashMap:

```.rust
extern crate yaml_rust;
use yaml_rust::yaml;

fn main() {
    let s =
"
foo:
    - list1
    - list2
bar:
    - 1
    - 2.0
";
    let docs = yaml::YamlLoader::load_from_str(s).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    println!("{:?}", doc);

    // Index access for map & array
    assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // Chained key/array access is checked and won't panic,
    // return BadValue if they are not exist.
    assert!(doc["INVALID_KEY"][100].is_badvalue());
}
```

Note that `yaml::Yaml` implements `Index<&'a str>` & `Index<usize>`:

* `Index<usize>` assumes the container is an Array
* `Index<&'a str>` assumes the container is a string to value Map
* otherwise, `Yaml::BadValue` is returned

If your document does not conform to this convention (e.g. map with
complex type key), you can use the `Yaml::as_XXX` family API to access your
documents.

## Features

* Pure Rust
* Ruby-like Array/Hash access API
* Low-level YAML events emission

## Specification Compliance

This implementation aims to provide YAML parser fully compatible with
the YAML 1.2 specification. The pasrser can correctly parse almost all
examples in the specification, except for the following known bugs:

* Empty plain scalar in certain contexts

However, the widely used library `libyaml` also fails to parse these examples,
so it may not be a huge problem for most users. 

## Goals

* Encoder
* Tag directive
* Alias while desearilization

## Contribution

Fork & PR on Github.
