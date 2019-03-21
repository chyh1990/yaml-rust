use super::parse_f64::parse_f64;
use derivative::Derivative;
use linked_hash_map::LinkedHashMap;
use scanner::Marker;
use std::ops::Index;
use std::{hash, string, vec};

pub trait YamlNode {
    type Child: YamlNode + Eq + hash::Hash + Ord;

    fn as_bool(&self) -> Option<bool>;
    fn as_f64(&self) -> Option<f64>;
    fn as_i64(&self) -> Option<i64>;
    fn as_str(&self) -> Option<&str>;
    fn as_hash(&self) -> Option<&LinkedHashMap<Self::Child, Self::Child>>;
    fn as_vec(&self) -> Option<&Vec<Self::Child>>;

    fn into_bool(self) -> Option<bool>;
    fn into_f64(self) -> Option<f64>;
    fn into_i64(self) -> Option<i64>;
    fn into_string(self) -> Option<String>;
    fn into_hash(self) -> Option<LinkedHashMap<Self::Child, Self::Child>>;
    fn into_vec(self) -> Option<Vec<Self::Child>>;

    fn is_null(&self) -> bool;
    fn is_badvalue(&self) -> bool;
    fn is_array(&self) -> bool;

    fn bad_value() -> &'static Self;
}

macro_rules! define_as (
    ($enum_name:ident, $name:ident, $t:ty, $yt:ident) => (
fn $name(&self) -> Option<$t> {
    match *self {
        $enum_name::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_ref (
    ($enum_name:ident, $name:ident, $t:ty, $yt:ident) => (
fn $name(&self) -> Option<$t> {
    match self {
        $enum_name::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_into (
    ($enum_name:ident, $name:ident, $t:ty, $yt:ident) => (
fn $name(self) -> Option<$t> {
    match self {
        $enum_name::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! yaml_enum (
    ($enum_name:ident, $child_type:ty, $bad_value:ident) => (
/// A YAML node is stored as this `Yaml` enumeration, which provides an easy way to
/// access your YAML document. The `YamlMarked` enumeration mirrors `Yaml`, but pairs each
/// child node with a source location marker.
///
/// # Examples
///
/// ```
/// use yaml_rust::{Yaml, YamlNode};
/// let foo = Yaml::from_str("-123"); // convert the string to the appropriate YAML type
/// assert_eq!(foo.as_i64().unwrap(), -123);
///
/// // iterate over an Array
/// let vec = Yaml::Array(vec![Yaml::Integer(1), Yaml::Integer(2)]);
/// for v in vec.as_vec().unwrap() {
///     assert!(v.as_i64().is_some());
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum $enum_name {
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
    Array(Vec<$child_type>),
    /// YAML hash, can be accessed as a `LinkedHashMap`.
    ///
    /// Insertion order will match the order of insertion into the map.
    Hash(LinkedHashMap<$child_type, $child_type>),
    /// Alias, not fully supported yet.
    Alias(usize),
    /// YAML null, e.g. `null` or `~`.
    Null,
    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    BadValue,
}

static $bad_value: $enum_name = $enum_name::BadValue;

impl YamlNode for $enum_name {
    type Child = $child_type;

    define_as!($enum_name, as_bool, bool, Boolean);
    define_as!($enum_name, as_i64, i64, Integer);

    define_as_ref!($enum_name, as_str, &str, String);
    define_as_ref!($enum_name, as_hash, &LinkedHashMap<Self::Child, Self::Child>, Hash);
    define_as_ref!($enum_name, as_vec, &Vec<Self::Child>, Array);

    define_into!($enum_name, into_bool, bool, Boolean);
    define_into!($enum_name, into_i64, i64, Integer);
    define_into!($enum_name, into_string, String, String);
    define_into!($enum_name, into_hash, LinkedHashMap<Self::Child, Self::Child>, Hash);
    define_into!($enum_name, into_vec, Vec<Self::Child>, Array);

    fn is_null(&self) -> bool {
        match self {
            $enum_name::Null => true,
            _ => false,
        }
    }

    fn is_badvalue(&self) -> bool {
        match self {
            $enum_name::BadValue => true,
            _ => false,
        }
    }

    fn is_array(&self) -> bool {
        match self {
            $enum_name::Array(_) => true,
            _ => false,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            $enum_name::Real(v) => parse_f64(v),
            _ => None,
        }
    }

    fn into_f64(self) -> Option<f64> {
        match self {
            $enum_name::Real(ref v) => parse_f64(v),
            _ => None,
        }
    }

    #[inline]
    fn bad_value() -> &'static Self {
        &$bad_value
    }
}
    );
);

yaml_enum!(Yaml, Yaml, BAD_VALUE_YAML);
yaml_enum!(YamlMarked, Node, BAD_VALUE_YAML_MARKED);

pub type Array = Vec<Yaml>;
pub type Hash = LinkedHashMap<Yaml, Yaml>;

pub type ArrayNode = Vec<Node>;
pub type HashNode = LinkedHashMap<Node, Node>;

/// A `Node` is a YAML AST node paired with a source location marker.
#[derive(Clone, Debug, Derivative, Ord, PartialOrd)]
#[derivative(Eq, Hash, PartialEq)]
pub struct Node(
    pub YamlMarked,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    pub Option<Marker>,
);

impl Node {
    pub fn marker(&self) -> Option<Marker> {
        self.1
    }

    pub fn value(&self) -> &YamlMarked {
        &self.0
    }

    pub fn into_value(self) -> YamlMarked {
        self.0
    }
}

macro_rules! node_method_ref (
    ($name:ident, $t:ty) => (
fn $name(&self) -> $t {
    self.value().$name()
}
    );
);

macro_rules! node_method_owned (
    ($name:ident, $t: ty) => (
fn $name(self) -> $t {
    self.into_value().$name()
}
    );
);

static BAD_VALUE_NODE: Node = Node(YamlMarked::BadValue, None);
impl YamlNode for Node {
    type Child = Node;

    node_method_ref!(as_bool, Option<bool>);
    node_method_ref!(as_f64, Option<f64>);
    node_method_ref!(as_i64, Option<i64>);
    node_method_ref!(as_str, Option<&str>);
    node_method_ref!(as_hash, Option<&LinkedHashMap<Self::Child, Self::Child>>);
    node_method_ref!(as_vec, Option<&Vec<Self::Child>>);

    node_method_owned!(into_bool, Option<bool>);
    node_method_owned!(into_i64, Option<i64>);
    node_method_owned!(into_f64, Option<f64>);
    node_method_owned!(into_string, Option<String>);
    node_method_owned!(into_hash, Option<LinkedHashMap<Self::Child, Self::Child>>);
    node_method_owned!(into_vec, Option<Vec<Self::Child>>);

    node_method_ref!(is_null, bool);
    node_method_ref!(is_badvalue, bool);
    node_method_ref!(is_array, bool);

    #[inline]
    fn bad_value() -> &'static Self {
        &BAD_VALUE_NODE
    }
}

impl From<YamlMarked> for Yaml {
    fn from(yaml: YamlMarked) -> Self {
        match yaml {
            YamlMarked::Real(s) => Yaml::Real(s),
            YamlMarked::Integer(i) => Yaml::Integer(i),
            YamlMarked::String(s) => Yaml::String(s),
            YamlMarked::Boolean(b) => Yaml::Boolean(b),
            YamlMarked::Array(v) => Yaml::Array(v.into_iter().map(|Node(y, _)| y.into()).collect()),
            YamlMarked::Hash(h) => Yaml::Hash(
                h.into_iter()
                    .map(|(Node(k, _), Node(v, _))| (k.into(), v.into()))
                    .collect(),
            ),
            YamlMarked::Alias(i) => Yaml::Alias(i),
            YamlMarked::Null => Yaml::Null,
            YamlMarked::BadValue => Yaml::BadValue,
        }
    }
}

impl From<Yaml> for Node {
    fn from(yaml: Yaml) -> Self {
        match yaml {
            Yaml::Real(s) => Node(YamlMarked::Real(s), None),
            Yaml::Integer(i) => Node(YamlMarked::Integer(i), None),
            Yaml::String(s) => Node(YamlMarked::String(s), None),
            Yaml::Boolean(b) => Node(YamlMarked::Boolean(b), None),
            Yaml::Array(v) => Node(
                YamlMarked::Array(v.into_iter().map(From::from).collect()),
                None,
            ),
            Yaml::Hash(h) => Node(
                YamlMarked::Hash(h.into_iter().map(|(k, v)| (k.into(), v.into())).collect()),
                None,
            ),
            Yaml::Alias(i) => Node(YamlMarked::Alias(i), None),
            Yaml::Null => Node(YamlMarked::Null, None),
            Yaml::BadValue => Node(YamlMarked::BadValue, None),
        }
    }
}

impl From<Node> for Yaml {
    fn from(node: Node) -> Self {
        node.into_value().into()
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl YamlMarked {
    // Not implementing FromStr because there is no possibility of Error.
    // This function falls back to YamlMarked::String if nothing else matches.
    pub fn from_str(v: &str) -> Self {
        if v.starts_with("0x") {
            let n = i64::from_str_radix(&v[2..], 16);
            if n.is_ok() {
                return YamlMarked::Integer(n.unwrap());
            }
        }
        if v.starts_with("0o") {
            let n = i64::from_str_radix(&v[2..], 8);
            if n.is_ok() {
                return YamlMarked::Integer(n.unwrap());
            }
        }
        if v.starts_with('+') && v[1..].parse::<i64>().is_ok() {
            return YamlMarked::Integer(v[1..].parse::<i64>().unwrap());
        }
        match v {
            "~" | "null" => YamlMarked::Null,
            "true" => YamlMarked::Boolean(true),
            "false" => YamlMarked::Boolean(false),
            _ if v.parse::<i64>().is_ok() => YamlMarked::Integer(v.parse::<i64>().unwrap()),
            // try parsing as f64
            _ if parse_f64(v).is_some() => YamlMarked::Real(v.to_owned()),
            _ => YamlMarked::String(v.to_owned()),
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl Yaml {
    pub fn from_str(v: &str) -> Self {
        YamlMarked::from_str(v).into()
    }
}

impl<'a> Index<&'a str> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = Yaml::String(idx.to_owned());
        match self.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&Yaml::bad_value()),
            None => &Yaml::bad_value(),
        }
    }
}

impl<'a> Index<&'a str> for YamlMarked {
    type Output = Node;

    fn index(&self, idx: &'a str) -> &Node {
        let key = Node(YamlMarked::String(idx.to_owned()), None);
        match self.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&Node::bad_value()),
            None => &Node::bad_value(),
        }
    }
}

impl<'a> Index<&'a str> for Node {
    type Output = Node;

    fn index(&self, idx: &'a str) -> &Node {
        self.value().index(idx)
    }
}

impl Index<usize> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: usize) -> &Yaml {
        if let Some(v) = self.as_vec() {
            v.get(idx).unwrap_or(&Yaml::bad_value())
        } else if let Some(v) = self.as_hash() {
            let key = Yaml::Integer(idx as i64);
            v.get(&key).unwrap_or(&Yaml::bad_value())
        } else {
            &Yaml::bad_value()
        }
    }
}

impl Index<usize> for YamlMarked {
    type Output = Node;

    fn index(&self, idx: usize) -> &Node {
        if let Some(v) = self.as_vec() {
            v.get(idx).unwrap_or(&Node::bad_value())
        } else if let Some(v) = self.as_hash() {
            let key = Node(YamlMarked::Integer(idx as i64), None);
            v.get(&key).unwrap_or(&Node::bad_value())
        } else {
            &Node::bad_value()
        }
    }
}

impl Index<usize> for Node {
    type Output = Node;

    fn index(&self, idx: usize) -> &Node {
        self.value().index(idx)
    }
}

macro_rules! define_into_iter (
    ($yaml_type:ty, $child_type:ty) => (
impl IntoIterator for $yaml_type {
    type Item = $child_type;
    type IntoIter = vec::IntoIter<$child_type>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().unwrap_or_else(Vec::new).into_iter()
    }
}
    );
);

define_into_iter!(Yaml, Yaml);
define_into_iter!(YamlMarked, Node);
define_into_iter!(Node, Node);
