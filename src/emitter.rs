use std::fmt;
use std::convert::From;
use yaml::*;

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
                            // complex key is not supported
                            Yaml::Array(_) | Yaml::Hash(_) => {
                                return Err(EmitError::BadHashmapKey);
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
                try!(escape_str(self.writer, v));
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
}
