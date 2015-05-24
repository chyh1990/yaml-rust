use std::collections::{HashMap, BTreeMap};
use std::string;
use regex::Regex;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub enum Yaml {
    I64(i64),
    //U64(u64),
    //F64(f64),
    String(string::String),
    Boolean(bool),
    Array(self::Array),
    Hash(self::Hash),
    Null,
}

pub type Array = Vec<Yaml>;
pub type Hash = BTreeMap<Yaml, Yaml>;

/// The errors that can arise while parsing a YAML stream.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidNumber,
    EOFWhileParsingObject,
    EOFWhileParsingArray,
    EOFWhileParsingValue,
    EOFWhileParsingString,
    KeyMustBeAString,
    ExpectedColon,
    TrailingCharacters,
    TrailingComma,
    InvalidEscape,
    InvalidUnicodeCodePoint,
    LoneLeadingSurrogateInHexEscape,
    UnexpectedEndOfHexEscape,
    UnrecognizedHex,
    NotFourDigit,
    NotUtf8,
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

// these regex are from libyaml-rust project
macro_rules! regex(
    ($s:expr) => (Regex::new($s).unwrap());
);
impl Yaml {
    define_as!(as_i64, i64, I64);
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

    pub fn as_f64(&self) -> Option<f64> {
        // XXX(chenyh) precompile me
        let float_pattern = regex!(r"^([-+]?)(\.[0-9]+|[0-9]+(\.[0-9]*)?([eE][-+]?[0-9]+)?)$");
        match *self {
            Yaml::String(ref v) if float_pattern.is_match(v) => {
                v.parse::<f64>().ok()
            },
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use parser::Parser;
    use yaml::Yaml;
    // #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let mut parser = Parser::new(s.chars());
        let out = parser.load().unwrap();
        //assert_eq!(out.as_hash().unwrap()[&Yaml::String("a".to_string())].as_i64().unwrap(), 1i64);
    }
}

