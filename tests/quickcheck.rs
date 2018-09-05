extern crate yaml_rust;
#[macro_use]
extern crate quickcheck;

use quickcheck::TestResult;
use yaml_rust::{Node, Yaml, YamlLoader, YamlEmitter};
use std::error::Error;

quickcheck! {
    fn test_check_weird_keys(xs: Vec<String>) -> TestResult {
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);

            let doc = Yaml(None, Node::Array(xs.into_iter().map(|x| Yaml(None, Node::String(x))).collect()));
            emitter.dump(&doc).unwrap();
        }
        if let Err(err) = YamlLoader::load_from_str(&out_str) {
            return TestResult::error(err.description());
        }
        TestResult::passed()
    }
}
