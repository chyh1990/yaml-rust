use std::collections::BTreeMap;
use std::ops::Index;
use std::string;
use std::str::FromStr;

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
    use parser::Parser;
    use yaml::Yaml;
    #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let mut parser = Parser::new(s.chars());
        let out = parser.load().unwrap();
        assert_eq!(out["a"].as_number::<i32>().unwrap(), 1);
        assert_eq!(out["b"].as_number::<f32>().unwrap(), 2.2f32);
        assert_eq!(out["c"][1].as_number::<i32>().unwrap(), 2);
        assert!(out["d"][0].is_badvalue());
        //assert_eq!(out.as_hash().unwrap()[&Yaml::String("a".to_string())].as_i64().unwrap(), 1i64);
    }
}

