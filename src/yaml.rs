use std::collections::{HashMap, BTreeMap};
use std::string;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Yaml {
    I64(i64),
    U64(u64),
    F64(f64),
    String(string::String),
    Boolean(bool),
    Array(self::Array),
    Hash(self::Hash),
    Null,
}

pub type Array = Vec<Yaml>;
pub type Hash = BTreeMap<string::String, Yaml>;

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
