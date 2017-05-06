use std::fmt;
use std::convert::From;
use yaml::Yaml;

#[derive(Copy, Clone, Debug)]
pub enum EmitError {
        FmtError(fmt::Error),
        BadHashmapKey,
}

impl From<fmt::Error> for EmitError {
    fn from(f: fmt::Error) -> Self {
        EmitError::FmtError(f)
    }
}

pub struct YamlEmitter<'a> {
    writer: &'a mut fmt::Write,
    best_indent: usize,

    level: isize,
}

pub type EmitResult = Result<(), EmitError>;

// from serialize::json
fn escape_str(wr: &mut fmt::Write, v: &str) -> Result<(), fmt::Error> {
    try!(wr.write_str("\""));

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write_str(&v[start..i]));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != v.len() {
        try!(wr.write_str(&v[start..]));
    }

    try!(wr.write_str("\""));
    Ok(())
}

impl<'a> YamlEmitter<'a> {
    pub fn new(writer: &'a mut fmt::Write) -> YamlEmitter {
        YamlEmitter {
            writer: writer,
            best_indent: 2,

            level: -1
        }
    }

    pub fn dump(&mut self, doc: &Yaml) -> EmitResult {
        // write DocumentStart
        try!(write!(self.writer, "---\n"));
        self.level = -1;
        self.emit_node(doc)
    }

    fn write_indent(&mut self) -> EmitResult {
        if self.level <= 0 { return Ok(()); }
        for _ in 0..self.level {
            for _ in 0..self.best_indent {
                try!(write!(self.writer, " "));
            }
        }
        Ok(())
    }

    fn emit_node_compact(&mut self, node: &Yaml) -> EmitResult {
        match *node {
            Yaml::Array(ref v) => {
                    try!(write!(self.writer, "["));
                    if self.level >= 0 {
                        try!(write!(self.writer, "+ "));
                    }
                    self.level += 1;
                    for (cnt, x) in v.iter().enumerate() {
                        try!(self.write_indent());
                        if cnt > 0 { try!(write!(self.writer, ", ")); }
                        try!(self.emit_node(x));
                    }
                    self.level -= 1;
                    try!(write!(self.writer, "]"));
                    Ok(())
            },
            Yaml::Hash(ref h) => {
                    try!(self.writer.write_str("{"));
                    self.level += 1;
                    for (cnt, (k, v)) in h.iter().enumerate() {
                        if cnt > 0 {
                            try!(write!(self.writer, ", "));
                        }
                        match *k {
                            // complex key is not supported
                            Yaml::Array(_) | Yaml::Hash(_) => {
                                return Err(EmitError::BadHashmapKey);
                            },
                            _ => { try!(self.emit_node(k)); }
                        }
                        try!(write!(self.writer, ": "));
                        try!(self.emit_node(v));
                    }
                    try!(self.writer.write_str("}"));
                    self.level -= 1;
                    Ok(())
            },
            _ => self.emit_node(node)
        }
    }

    fn emit_node(&mut self, node: &Yaml) -> EmitResult {
        match *node {
            Yaml::Array(ref v) => {
                if v.is_empty() {
                    try!(write!(self.writer, "[]"));
                    Ok(())
                } else {
                    if self.level >= 0 {
                        try!(write!(self.writer, "\n"));
                    }
                    self.level += 1;
                    for (cnt, x) in v.iter().enumerate() {
                        if cnt > 0 {
                            try!(write!(self.writer, "\n"));
                        }
                        try!(self.write_indent());
                        try!(write!(self.writer, "- "));
                        try!(self.emit_node(x));
                    }
                    self.level -= 1;
                    Ok(())
                }
            },
            Yaml::Hash(ref h) => {
                if h.is_empty() {
                    try!(self.writer.write_str("{}"));
                    Ok(())
                } else {
                    if self.level >= 0 {
                        try!(write!(self.writer, "\n"));
                    }
                    self.level += 1;
                    for (cnt, (k, v)) in h.iter().enumerate() {
                        if cnt > 0 {
                            try!(write!(self.writer, "\n"));
                        }
                        try!(self.write_indent());
                        match *k {
                            Yaml::Array(_) | Yaml::Hash(_) => {
                                try!(self.emit_node_compact(k));
                                //return Err(EmitError::BadHashmapKey);
                            },
                            _ => { try!(self.emit_node(k)); }
                        }
                        try!(write!(self.writer, ": "));
                        try!(self.emit_node(v));
                    }
                    self.level -= 1;
                    Ok(())
                }
            },
            Yaml::String(ref v) => {
                if need_quotes(v) {
                    try!(escape_str(self.writer, v));
                }
                else {
                    try!(write!(self.writer, "{}", v));
                }
                Ok(())
            },
            Yaml::Boolean(v) => {
                if v {
                    try!(self.writer.write_str("true"));
                } else {
                    try!(self.writer.write_str("false"));
                }
                Ok(())
            },
            Yaml::Integer(v) => {
                try!(write!(self.writer, "{}", v));
                Ok(())
            },
            Yaml::Real(ref v) => {
                try!(write!(self.writer, "{}", v));
                Ok(())
            },
            Yaml::Null | Yaml::BadValue => {
                try!(write!(self.writer, "~"));
                Ok(())
            },
            // XXX(chenyh) Alias
            _ => { Ok(()) }
        }
    }
}

/// Check if the string requires quoting.
/// Strings containing any of the following characters must be quoted.
/// :, {, }, [, ], ,, &, *, #, ?, |, -, <, >, =, !, %, @, `
///
/// If the string contains any of the following control characters, it must be escaped with double quotes:
/// \0, \x01, \x02, \x03, \x04, \x05, \x06, \a, \b, \t, \n, \v, \f, \r, \x0e, \x0f, \x10, \x11, \x12, \x13, \x14, \x15, \x16, \x17, \x18, \x19, \x1a, \e, \x1c, \x1d, \x1e, \x1f, \N, \_, \L, \P
///
/// Finally, there are other cases when the strings must be quoted, no matter if you're using single or double quotes:
/// * When the string is true or false (otherwise, it would be treated as a boolean value);
/// * When the string is null or ~ (otherwise, it would be considered as a null value);
/// * When the string looks like a number, such as integers (e.g. 2, 14, etc.), floats (e.g. 2.6, 14.9) and exponential numbers (e.g. 12e7, etc.) (otherwise, it would be treated as a numeric value);
/// * When the string looks like a date (e.g. 2014-12-31) (otherwise it would be automatically converted into a Unix timestamp).
fn need_quotes(string: &str) -> bool {
    fn need_quotes_spaces(string: &str) -> bool {
        string.starts_with(' ')
            || string.ends_with(' ')
    }

    string == ""
    || need_quotes_spaces(string)
    || string.contains(|character: char| {
        match character {
            ':' | '{' | '}' | '[' | ']' | ',' | '&' | '*' | '#' | '?' | '|' | '-' | '<' | '>' | '=' | '!' | '%' | '@' | '`' | '\\' | '\0' ... '\x06' | '\t' | '\n' | '\r' | '\x0e' ... '\x1a' | '\x1c' ... '\x1f' => true,
            _ => false,
        }
    })
    || string == "true"
    || string == "false"
    || string == "null"
    || string == "~"
    || string.starts_with('.')
    || string.parse::<i64>().is_ok()
    || string.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml::*;

    #[test]
    fn test_emit_simple() {
        let s = "
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
    - []
    - {}
a5: 'single_quoted'
a6: \"double_quoted\"
a7: 你好
'key 1': \"ddd\\\tbbb\"
";


        let docs = YamlLoader::load_from_str(&s).unwrap();
        let doc = &docs[0];
        let mut writer = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut writer);
            emitter.dump(doc).unwrap();
        }
        let docs_new = YamlLoader::load_from_str(&s).unwrap();
        let doc_new = &docs_new[0];

        assert_eq!(doc, doc_new);
    }

    #[test]
    fn test_emit_complex() {
        let s = r#"
cataloge:
  product: &coffee   { name: Coffee,    price: 2.5  ,  unit: 1l  }
  product: &cookies  { name: Cookies!,  price: 3.40 ,  unit: 400g}

products:
  *coffee:
    amount: 4
  *cookies:
    amount: 4
  [1,2,3,4]:
    array key
  2.4:
    real key
  true:
    bool key
  {}:
    empty hash key
            "#;
        let docs = YamlLoader::load_from_str(&s).unwrap();
        let doc = &docs[0];
        let mut writer = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut writer);
            emitter.dump(doc).unwrap();
        }
        let docs_new = YamlLoader::load_from_str(&s).unwrap();
        let doc_new = &docs_new[0];
        assert_eq!(doc, doc_new);
    }

    #[test]
    fn test_emit_avoid_quotes() {
        let s = r#"---
a7: 你好
boolean: "true"
boolean2: "false"
date: "2014-12-31"
empty_string: ""
empty_string1: " "
empty_string2: "    a"
empty_string3: "    a "
exp: "12e7"
field: ":"
field2: "{"
field3: "\\"
field4: "\n"
float: "2.6"
int: "4"
nullable: "null"
nullable2: "~"
products: 
  "*coffee": 
    amount: 4
  "*cookies": 
    amount: 4
  ".milk":
    amount: 1
  "2.4": real key
  "[1,2,3,4]": array key
  "true": bool key
  "{}": empty hash key
x: test
y: string with spaces"#;

        let docs = YamlLoader::load_from_str(&s).unwrap();
        let doc = &docs[0];
        let mut writer = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut writer);
            emitter.dump(doc).unwrap();
        }

        assert_eq!(s, writer);
    }
}
