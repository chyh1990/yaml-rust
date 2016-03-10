use std::collections::BTreeMap;
use std::ops::Index;
use std::string;
use std::i64;
use std::str::FromStr;
use std::mem;
use parser::*;
use scanner::{TScalarStyle, ScanError, TokenType};

/// A YAML node is stored as this `Yaml` enumeration, which provides an easy way to
/// access your YAML document.
///
/// # Examples
///
/// ```
/// use yaml_rust::Yaml;
/// let foo = Yaml::from_str("-123"); // convert the string to the appropriate YAML type
/// assert_eq!(foo.as_i64().unwrap(), -123);
///
/// // iterate over an Array
/// let vec = Yaml::Array(vec![Yaml::Integer(1), Yaml::Integer(2)]);
/// for v in vec.as_vec().unwrap() {
///     assert!(v.as_i64().is_some());
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub enum Yaml {
    /// Float types are stored as String and parsed on demand.
    /// Note that f64 does NOT implement Eq trait and can NOT be stored in BTreeMap.
    Real(string::String),
    /// YAML int is stored as i64.
    Integer(i64),
    /// YAML scalar.
    String(string::String),
    /// YAML bool, e.g. `true` or `false`.
    Boolean(bool),
    /// YAML array, can be accessed as a `Vec`.
    Array(self::Array),
    /// YAML hash, can be accessed as a `BTreeMap`.
    Hash(self::Hash),
    /// Alias, not fully supported yet.
    Alias(usize),
    /// YAML null, e.g. `null` or `~`.
    Null,
    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    BadValue,
}

pub type Array = Vec<Yaml>;
pub type Hash = BTreeMap<Yaml, Yaml>;

pub struct YamlLoader {
    docs: Vec<Yaml>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(Yaml, usize)>,
    key_stack: Vec<Yaml>,
    anchor_map: BTreeMap<usize, Yaml>,
}

impl EventReceiver for YamlLoader {
    fn on_event(&mut self, ev: &Event) {
        // println!("EV {:?}", ev);
        match *ev {
            Event::DocumentStart => {
                // do nothing
            },
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(Yaml::BadValue),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!()
                }
            },
            Event::SequenceStart(aid) => {
                self.doc_stack.push((Yaml::Array(Vec::new()), aid));
            },
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            },
            Event::MappingStart(aid) => {
                self.doc_stack.push((Yaml::Hash(Hash::new()), aid));
                self.key_stack.push(Yaml::BadValue);
            },
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            },
            Event::Scalar(ref v, style, aid, ref tag) => {
                let node = if style != TScalarStyle::Plain {
                    Yaml::String(v.clone())
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = *tag {
                    // XXX tag:yaml.org,2002:
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => {
                                // "true" or "false"
                                match v.parse::<bool>() {
                                    Err(_) => Yaml::BadValue,
                                    Ok(v) => Yaml::Boolean(v)
                                }
                            },
                            "int" => {
                                match v.parse::<i64>() {
                                    Err(_) => Yaml::BadValue,
                                    Ok(v) => Yaml::Integer(v)
                                }
                            },
                            "float" => {
                                match v.parse::<f64>() {
                                    Err(_) => Yaml::BadValue,
                                    Ok(_) => Yaml::Real(v.clone())
                                }
                            },
                            "null" => {
                                match v.as_ref() {
                                    "~" | "null" => Yaml::Null,
                                    _ => Yaml::BadValue,
                                }
                            }
                            _  => Yaml::String(v.clone()),
                        }
                    } else {
                        Yaml::String(v.clone())
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    Yaml::from_str(v.as_ref())
                };

                self.insert_new_node((node, aid));
            },
            Event::Alias(id) => {
                let n = match self.anchor_map.get(&id) {
                    Some(v) => v.clone(),
                    None => Yaml::BadValue,
                };
                self.insert_new_node((n, 0));
            }
            _ => { /* ignore */ }
        }
        // println!("DOC {:?}", self.doc_stack);
    }
}

impl YamlLoader {
    fn insert_new_node(&mut self, node: (Yaml, usize)) {
        // valid anchor id starts from 1
        if node.1 > 0 {
            self.anchor_map.insert(node.1, node.0.clone());
        }
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (Yaml::Array(ref mut v), _) => v.push(node.0),
                (Yaml::Hash(ref mut h), _) => {
                    let mut cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = Yaml::BadValue;
                        mem::swap(&mut newkey, cur_key);
                        h.insert(newkey, node.0);
                    }
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<Yaml>, ScanError>{
        let mut loader = YamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
            anchor_map: BTreeMap::new(),
        };
        let mut parser = Parser::new(source.chars());
        try!(parser.load(&mut loader, true));
        Ok(loader.docs)
    }
}

macro_rules! define_as (
    ($name:ident, $t:ident, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        Yaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_ref (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        Yaml::$yt(ref v) => Some(v),
        _ => None
    }
}
    );
);

impl Yaml {
    define_as!(as_bool, bool, Boolean);
    define_as!(as_i64, i64, Integer);

    define_as_ref!(as_str, &str, String);
    define_as_ref!(as_hash, &Hash, Hash);
    define_as_ref!(as_vec, &Array, Array);

    pub fn is_null(&self) -> bool {
        match *self {
            Yaml::Null => true,
            _ => false
        }
    }

    pub fn is_badvalue(&self) -> bool {
        match *self {
            Yaml::BadValue => true,
            _ => false
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Yaml::Real(ref v) => {
                v.parse::<f64>().ok()
            },
            _ => None
        }
    }
}

#[cfg_attr(feature="clippy", allow(should_implement_trait))]
impl Yaml {
    // Not implementing FromStr because there is no possibility of Error.
    // This function falls back to Yaml::String if nothing else matches.
    pub fn from_str(v: &str) -> Yaml {
        if v.starts_with("0x") {
            let n = i64::from_str_radix(&v[2..], 16);
            if n.is_ok() {
                return Yaml::Integer(n.unwrap());
            }
        }
        if v.starts_with("0o") {
            let n = i64::from_str_radix(&v[2..], 8);
            if n.is_ok() {
                return Yaml::Integer(n.unwrap());
            }
        }
        if v.starts_with('+') && v[1..].parse::<i64>().is_ok() {
            return Yaml::Integer(v[1..].parse::<i64>().unwrap());
        }
        match v {
            "~" | "null" => Yaml::Null,
            "true" => Yaml::Boolean(true),
            "false" => Yaml::Boolean(false),
            _ if v.parse::<i64>().is_ok() => Yaml::Integer(v.parse::<i64>().unwrap()),
            // try parsing as f64
            _ if v.parse::<f64>().is_ok() => Yaml::Real(v.to_owned()),
            _ => Yaml::String(v.to_owned())
        }
    }
}

static BAD_VALUE: Yaml = Yaml::BadValue;
impl<'a> Index<&'a str> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = Yaml::String(idx.to_owned());
        match self.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&BAD_VALUE),
            None => &BAD_VALUE
        }
    }
}

impl Index<usize> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: usize) -> &Yaml {
        match self.as_vec() {
            Some(v) => v.get(idx).unwrap_or(&BAD_VALUE),
            None => &BAD_VALUE
        }
    }
}

#[cfg(test)]
mod test {
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
".to_owned();
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a7"].as_str().unwrap(), "你好");
    }

    #[test]
    fn test_multi_doc() {
        let s =
"
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
        let s =
"
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
        let s =
"
a1: &DEFAULT
    b1: 4
    b2: *DEFAULT
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a1"]["b2"], Yaml::BadValue);

    }


    #[test]
    fn test_plain_datatype() {
        let s =
"
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
}
