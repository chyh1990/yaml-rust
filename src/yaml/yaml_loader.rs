use super::ast::{ArrayNode, HashNode, Node, Yaml, YamlMarked, YamlNode};
use super::parse_f64::parse_f64;
use parser::*;
use scanner::{Marker, ScanError, TScalarStyle, TokenType};
use std::collections::BTreeMap;
use std::{i64, mem};

pub struct YamlLoader {
    docs: Vec<Node>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(Node, usize)>,
    key_stack: Vec<Node>,
    anchor_map: BTreeMap<usize, YamlMarked>,
}

impl MarkedEventReceiver for YamlLoader {
    fn on_event(&mut self, ev: Event, marker: Marker) {
        // println!("EV {:?}", ev);
        match ev {
            Event::DocumentStart => {
                // do nothing
            }
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(Node(YamlMarked::BadValue, Some(marker))),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid) => {
                self.doc_stack
                    .push((Node(YamlMarked::Array(ArrayNode::new()), Some(marker)), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::MappingStart(aid) => {
                self.doc_stack
                    .push((Node(YamlMarked::Hash(HashNode::new()), Some(marker)), aid));
                self.key_stack
                    .push(Node(YamlMarked::BadValue, Some(marker)));
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::Scalar(v, style, aid, tag) => {
                let yaml = if style != TScalarStyle::Plain {
                    YamlMarked::String(v)
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = tag {
                    // XXX tag:yaml.org,2002:
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => {
                                // "true" or "false"
                                match v.parse::<bool>() {
                                    Err(_) => YamlMarked::BadValue,
                                    Ok(v) => YamlMarked::Boolean(v),
                                }
                            }
                            "int" => match v.parse::<i64>() {
                                Err(_) => YamlMarked::BadValue,
                                Ok(v) => YamlMarked::Integer(v),
                            },
                            "float" => match parse_f64(&v) {
                                Some(_) => YamlMarked::Real(v),
                                None => YamlMarked::BadValue,
                            },
                            "null" => match v.as_ref() {
                                "~" | "null" => YamlMarked::Null,
                                _ => YamlMarked::BadValue,
                            },
                            _ => YamlMarked::String(v),
                        }
                    } else {
                        YamlMarked::String(v)
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    YamlMarked::from_str(&v)
                };
                let node = Node(yaml, Some(marker));
                self.insert_new_node((node, aid));
            }
            Event::Alias(id) => {
                let n = match self.anchor_map.get(&id) {
                    Some(v) => Node(v.clone(), Some(marker)),
                    None => Node(YamlMarked::BadValue, Some(marker)),
                };
                self.insert_new_node((n, 0));
            }
            _ => { /* ignore */ }
        }
        // println!("DOC {:?}", self.doc_stack);
    }
}

impl YamlLoader {
    fn insert_new_node(&mut self, node: (Node, usize)) {
        // valid anchor id starts from 1
        if node.1 > 0 {
            self.anchor_map.insert(node.1, (node.0).0.clone());
        }
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (Node(YamlMarked::Array(ref mut v), _), _) => v.push(node.0),
                (Node(YamlMarked::Hash(ref mut h), _), _) => {
                    let cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.0.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = Node(YamlMarked::BadValue, (node.0).marker());
                        mem::swap(&mut newkey, cur_key);
                        h.insert(newkey, node.0);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<Yaml>, ScanError> {
        let ast = Self::load_from_str_with_markers(source)?;
        Ok(ast.into_iter().map(|n| n.into()).collect())
    }

    pub fn load_from_str_with_markers(source: &str) -> Result<Vec<Node>, ScanError> {
        let mut loader = YamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
            anchor_map: BTreeMap::new(),
        };
        let mut parser = Parser::new(source.chars());
        parser.load(&mut loader, true)?;
        Ok(loader.docs)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::float_cmp))]
#[cfg(test)]
mod test {
    use std::f64;
    use yaml::*;

    #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a"].as_i64().unwrap(), 1i64);
        assert_eq!(doc["b"].as_f64().unwrap(), 2.2f64);
        assert_eq!(doc["c"][1].as_i64().unwrap(), 2i64);
        assert!(doc["d"][0].is_badvalue());
    }

    #[test]
    fn test_empty_doc() {
        let s: String = "".to_owned();
        YamlLoader::load_from_str(&s).unwrap();
        let s: String = "---".to_owned();
        assert_eq!(YamlLoader::load_from_str(&s).unwrap()[0], Yaml::Null);
    }

    #[test]
    fn test_parser() {
        let s: String = "
# comment
a0 bb: val
a1:
    b1: 4
    b2: d
a2: 4 # i'm comment
a3: [1, 2, 3]
a4:
    - - a1
      - a2
    - 2
a5: 'single_quoted'
a6: \"double_quoted\"
a7: 你好
"
        .to_owned();
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a7"].as_str().unwrap(), "你好");
    }

    #[test]
    fn test_multi_doc() {
        let s = "
'a scalar'
---
'a scalar'
---
'a scalar'
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_anchor() {
        let s = "
a1: &DEFAULT
    b1: 4
    b2: d
a2: *DEFAULT
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a2"]["b1"].as_i64().unwrap(), 4);
    }

    #[test]
    fn test_bad_anchor() {
        let s = "
a1: &DEFAULT
    b1: 4
    b2: *DEFAULT
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a1"]["b2"], Yaml::BadValue);
    }

    #[test]
    fn test_github_27() {
        // https://github.com/chyh1990/yaml-rust/issues/27
        let s = "&a";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.as_str().unwrap(), "");
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cyclomatic_complexity))]
    #[test]
    fn test_plain_datatype() {
        let s = "
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- ~
- null
- true
- false
- !!str 0
- !!int 100
- !!float 2
- !!null ~
- !!bool true
- !!bool false
- 0xFF
# bad values
- !!int string
- !!float string
- !!bool null
- !!null val
- 0o77
- [ 0xF, 0xF ]
- +12345
- [ true, false ]
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];

        assert_eq!(doc[0].as_str().unwrap(), "string");
        assert_eq!(doc[1].as_str().unwrap(), "string");
        assert_eq!(doc[2].as_str().unwrap(), "string");
        assert_eq!(doc[3].as_i64().unwrap(), 123);
        assert_eq!(doc[4].as_i64().unwrap(), -321);
        assert_eq!(doc[5].as_f64().unwrap(), 1.23);
        assert_eq!(doc[6].as_f64().unwrap(), -1e4);
        assert!(doc[7].is_null());
        assert!(doc[8].is_null());
        assert_eq!(doc[9].as_bool().unwrap(), true);
        assert_eq!(doc[10].as_bool().unwrap(), false);
        assert_eq!(doc[11].as_str().unwrap(), "0");
        assert_eq!(doc[12].as_i64().unwrap(), 100);
        assert_eq!(doc[13].as_f64().unwrap(), 2.0);
        assert!(doc[14].is_null());
        assert_eq!(doc[15].as_bool().unwrap(), true);
        assert_eq!(doc[16].as_bool().unwrap(), false);
        assert_eq!(doc[17].as_i64().unwrap(), 255);
        assert!(doc[18].is_badvalue());
        assert!(doc[19].is_badvalue());
        assert!(doc[20].is_badvalue());
        assert!(doc[21].is_badvalue());
        assert_eq!(doc[22].as_i64().unwrap(), 63);
        assert_eq!(doc[23][0].as_i64().unwrap(), 15);
        assert_eq!(doc[23][1].as_i64().unwrap(), 15);
        assert_eq!(doc[24].as_i64().unwrap(), 12345);
        assert!(doc[25][0].as_bool().unwrap());
        assert!(!doc[25][1].as_bool().unwrap());
    }

    #[test]
    fn test_bad_hypen() {
        // See: https://github.com/chyh1990/yaml-rust/issues/23
        let s = "{-";
        assert!(YamlLoader::load_from_str(&s).is_err());
    }

    #[test]
    fn test_issue_65() {
        // See: https://github.com/chyh1990/yaml-rust/issues/65
        let b = "\n\"ll\\\"ll\\\r\n\"ll\\\"ll\\\r\r\r\rU\r\r\rU";
        assert!(YamlLoader::load_from_str(&b).is_err());
    }

    #[test]
    fn test_bad_docstart() {
        assert!(YamlLoader::load_from_str("---This used to cause an infinite loop").is_ok());
        assert_eq!(
            YamlLoader::load_from_str("----"),
            Ok(vec![Yaml::String(String::from("----"))])
        );
        assert_eq!(
            YamlLoader::load_from_str("--- #here goes a comment"),
            Ok(vec![Yaml::Null])
        );
        assert_eq!(
            YamlLoader::load_from_str("---- #here goes a comment"),
            Ok(vec![Yaml::String(String::from("----"))])
        );
    }

    #[test]
    fn test_plain_datatype_with_into_methods() {
        let s = "
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- true
- false
- !!str 0
- !!int 100
- !!float 2
- !!bool true
- !!bool false
- 0xFF
- 0o77
- +12345
- -.INF
- .NAN
- !!float .INF
";
        let mut out = YamlLoader::load_from_str(&s).unwrap().into_iter();
        let mut doc = out.next().unwrap().into_iter();

        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), 123);
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), -321);
        assert_eq!(doc.next().unwrap().into_f64().unwrap(), 1.23);
        assert_eq!(doc.next().unwrap().into_f64().unwrap(), -1e4);
        assert_eq!(doc.next().unwrap().into_bool().unwrap(), true);
        assert_eq!(doc.next().unwrap().into_bool().unwrap(), false);
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "0");
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), 100);
        assert_eq!(doc.next().unwrap().into_f64().unwrap(), 2.0);
        assert_eq!(doc.next().unwrap().into_bool().unwrap(), true);
        assert_eq!(doc.next().unwrap().into_bool().unwrap(), false);
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), 255);
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), 63);
        assert_eq!(doc.next().unwrap().into_i64().unwrap(), 12345);
        assert_eq!(doc.next().unwrap().into_f64().unwrap(), f64::NEG_INFINITY);
        assert!(doc.next().unwrap().into_f64().is_some());
        assert_eq!(doc.next().unwrap().into_f64().unwrap(), f64::INFINITY);
    }

    #[test]
    fn test_hash_order() {
        let s = "---
b: ~
a: ~
c: ~
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let first = out.into_iter().next().unwrap();
        let mut iter = first.into_hash().unwrap().into_iter();
        assert_eq!(
            Some((Yaml::String("b".to_owned()), Yaml::Null)),
            iter.next()
        );
        assert_eq!(
            Some((Yaml::String("a".to_owned()), Yaml::Null)),
            iter.next()
        );
        assert_eq!(
            Some((Yaml::String("c".to_owned()), Yaml::Null)),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_integer_key() {
        let s = "
0:
    important: true
1:
    important: false
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let first = out.into_iter().next().unwrap();
        assert_eq!(first[0]["important"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_indentation_equality() {
        let four_spaces = YamlLoader::load_from_str(
            r#"
hash:
    with:
        indentations
"#,
        )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

        let two_spaces = YamlLoader::load_from_str(
            r#"
hash:
  with:
    indentations
"#,
        )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

        let one_space = YamlLoader::load_from_str(
            r#"
hash:
 with:
  indentations
"#,
        )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

        let mixed_spaces = YamlLoader::load_from_str(
            r#"
hash:
     with:
               indentations
"#,
        )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

        assert_eq!(four_spaces, two_spaces);
        assert_eq!(two_spaces, one_space);
        assert_eq!(four_spaces, mixed_spaces);
    }

    #[test]
    fn test_two_space_indentations() {
        // https://github.com/kbknapp/clap-rs/issues/965

        let s = r#"
subcommands:
  - server:
    about: server related commands
subcommands2:
  - server:
      about: server related commands
subcommands3:
 - server:
    about: server related commands
            "#;

        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out.into_iter().next().unwrap();

        println!("{:#?}", doc);
        assert_eq!(doc["subcommands"][0]["server"], Yaml::Null);
        assert!(doc["subcommands2"][0]["server"].as_hash().is_some());
        assert!(doc["subcommands3"][0]["server"].as_hash().is_some());
    }

    #[test]
    fn test_recursion_depth_check_objects() {
        let s = "{a:".repeat(10_000) + &"}".repeat(10_000);
        assert!(YamlLoader::load_from_str(&s).is_err());
    }

    #[test]
    fn test_recursion_depth_check_arrays() {
        let s = "[".repeat(10_000) + &"]".repeat(10_000);
        assert!(YamlLoader::load_from_str(&s).is_err());
    }
}
