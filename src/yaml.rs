use std::collections::BTreeMap;
use std::ops::Index;
use std::string;
use std::str::FromStr;
use std::mem;
use parser::*;
use scanner::{TScalarStyle, ScanError};

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub enum Yaml {
    /// number types are stored as String, and parsed on demand.
    Number(string::String),
    String(string::String),
    Boolean(bool),
    Array(self::Array),
    Hash(self::Hash),
    Null,
    /// Access non-exist node by Index trait will return BadValue.
    /// This simplifies error handling of user.
    BadValue,
}

pub type Array = Vec<Yaml>;
pub type Hash = BTreeMap<Yaml, Yaml>;

pub struct YamlLoader {
    docs: Vec<Yaml>,
    // states
    doc_stack: Vec<Yaml>,
    key_stack: Vec<Yaml>,
}

impl EventReceiver for YamlLoader {
    fn on_event(&mut self, ev: &Event) {
        println!("EV {:?}", ev);
        match *ev {
            Event::DocumentStart => {
                // do nothing
            },
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(Yaml::BadValue),
                    1 => self.docs.push(self.doc_stack.pop().unwrap()),
                    _ => unreachable!()
                }
            },
            Event::SequenceStart => {
                self.doc_stack.push(Yaml::Array(Vec::new()));
            },
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            },
            Event::MappingStart => {
                self.doc_stack.push(Yaml::Hash(Hash::new()));
                self.key_stack.push(Yaml::BadValue);
            },
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            },
            Event::Scalar(ref v, style) => {
                let node = if style != TScalarStyle::Plain {
                    Yaml::String(v.clone())
                } else {
                    match v.as_ref() {
                        "~" => Yaml::Null,
                        "true" => Yaml::Boolean(true),
                        "false" => Yaml::Boolean(false),
                        // try parsing as f64
                        _ if v.parse::<f64>().is_ok() => Yaml::Number(v.clone()),
                        _ => Yaml::String(v.clone())
                    }
                };

                self.insert_new_node(node);
            },
            _ => { /* ignore */ }
        }
        // println!("DOC {:?}", self.doc_stack);
    }
}

impl YamlLoader {
    fn insert_new_node(&mut self, node: Yaml) {
        if !self.doc_stack.is_empty() {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                Yaml::Array(ref mut v) => v.push(node),
                Yaml::Hash(ref mut h) => {
                    let mut cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node;
                    // current node is a value
                    } else {
                        let mut newkey = Yaml::BadValue;
                        mem::swap(&mut newkey, cur_key);
                        h.insert(newkey, node);
                    }
                },
                _ => unreachable!(),
            }
        } else {
            self.doc_stack.push(node);
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<Yaml>, ScanError>{
        let mut loader = YamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
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

    pub fn as_number<T: FromStr>(&self) -> Option<T> {
        match *self {
            Yaml::Number(ref v) => {
                v.parse::<T>().ok()
            },
            _ => None
        }
    }

    pub fn from_str(s: &str) -> Yaml {
        Yaml::String(s.to_string())
    }
}

static BAD_VALUE: Yaml = Yaml::BadValue;
impl<'a> Index<&'a str> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = Yaml::String(idx.to_string());
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
        assert_eq!(doc["a"].as_number::<i32>().unwrap(), 1);
        assert_eq!(doc["b"].as_number::<f32>().unwrap(), 2.2f32);
        assert_eq!(doc["c"][1].as_number::<i32>().unwrap(), 2);
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
".to_string();
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        println!("DOC {:?}", doc);
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

}

