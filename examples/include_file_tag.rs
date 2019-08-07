extern crate yaml_rust;

mod dump_yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::env;
use yaml_rust::yaml;
use yaml_rust::scanner;

struct IncludeParser<'a> {
    root: &'a Path
}

impl<'a> IncludeParser<'a> {
    fn new(root: &'a Path) -> IncludeParser {
        IncludeParser {
            root: root
        }
    }
}

impl<'a> yaml::YamlScalarParser for IncludeParser<'a> {
    fn parse_scalar(&self, tag: &scanner::TokenType, value: &str) -> Option<yaml::Yaml> {
        if let scanner::TokenType::Tag(ref handle, ref suffix) = *tag {
            if (*handle == "!" || *handle == "yaml-rust.include.prefix") && *suffix == "include" {
                let mut content = String::new();
                return Some(match File::open(self.root.join(value)){
                    Ok(mut f) => {
                        let _ = f.read_to_string(&mut content);
                        let mut loader = yaml::YamlLoader::new();
                        loader.register_scalar_parser(self);
                        match loader.parse_from_str(&content.to_owned()) {
                            Ok(mut docs) => docs.pop().unwrap(),
                            Err(_) => yaml::Yaml::BadValue
                        }
                    }
                    Err(_) => yaml::Yaml::BadValue
                })
            }
        }
        None
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut f = File::open(&args[1]).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let p = env::current_dir().unwrap();
    let parser = IncludeParser::new(p.as_path());
    let mut loader = yaml::YamlLoader::new();
    loader.register_scalar_parser(&parser);

    let docs = loader.parse_from_str(&s).unwrap();
    for doc in &docs {
        println!("---");
        dump_yaml::dump_node(doc, 0);
    }
}