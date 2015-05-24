use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum TEncoding {
    Utf8
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum TScalarStyle {
    Any,
    Plain,
    SingleQuoted,
    DoubleQuoted,

    Literal,
    Foled
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub struct Marker {
    index: usize,
    line: usize,
    col: usize,
}

impl Marker {
    fn new(index: usize, line: usize, col: usize) -> Marker {
        Marker {
            index: index,
            line: line,
            col: col
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ScanError {
    mark: Marker,
    info: String,
}

impl ScanError {
    pub fn new(loc: Marker, info: &str) -> ScanError {
        ScanError {
            mark: loc,
            info: info.to_string()
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum TokenType {
    NoToken,
    StreamStartToken(TEncoding),
    StreamEndToken,
    VersionDirectiveToken,
    TagDirectiveToken,
    DocumentStartToken,
    DocumentEndToken,
    BlockSequenceStartToken,
    BlockMappingStartToken,
    BlockEndToken,
    FlowSequenceStartToken,
    FlowSequenceEndToken,
    FlowMappingStartToken,
    FlowMappingEndToken,
    BlockEntryToken,
    FlowEntryToken,
    KeyToken,
    ValueToken,
    AliasToken,
    AnchorToken,
    TagToken,
    ScalarToken(TScalarStyle, String)
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Token(pub Marker, pub TokenType);

#[derive(Clone, PartialEq, Debug, Eq)]
struct SimpleKey {
    possible: bool,
    required: bool,
    token_number: usize,
    mark: Marker,
}

impl SimpleKey {
    fn new(mark: Marker) -> SimpleKey {
        SimpleKey {
            possible: false,
            required: false,
            token_number: 0,
            mark: mark,
        }
    }
}

#[derive(Debug)]
pub struct Scanner<T> {
    rdr: T,
    mark: Marker,
    tokens: VecDeque<Token>,
    buffer: VecDeque<char>,

    stream_start_produced: bool,
    stream_end_produced: bool,
    simple_key_allowed: bool,
    simple_keys: Vec<SimpleKey>,
    indent: isize,
    indents: Vec<isize>,
    flow_level: usize,
    tokens_parsed: usize,
    token_available: bool,
}

impl<T: Iterator<Item=char>> Iterator for Scanner<T> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        match self.next_token() {
            Ok(tok) => tok,
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        }
    }
}

fn is_z(c: char) -> bool {
    c == '\0'
}
fn is_break(c: char) -> bool {
    c == '\n' || c == '\r'
}
fn is_breakz(c: char) -> bool {
    is_break(c) || is_z(c)
}
fn is_blank(c: char) -> bool {
    c == ' ' || c == '\t'
}
fn is_blankz(c: char) -> bool {
    is_blank(c) || is_breakz(c)
}

pub type ScanResult = Result<(), ScanError>;

impl<T: Iterator<Item=char>> Scanner<T> {
    /// Creates the YAML tokenizer.
    pub fn new(rdr: T) -> Scanner<T> {
        Scanner {
            rdr: rdr,
            buffer: VecDeque::new(),
            mark: Marker::new(0, 1, 0),
            tokens: VecDeque::new(),

            stream_start_produced: false,
            stream_end_produced: false,
            simple_key_allowed: true,
            simple_keys: Vec::new(),
            indent: -1,
            indents: Vec::new(),
            flow_level: 0,
            tokens_parsed: 0,
            token_available: false,
        }
    }

    fn lookahead(&mut self, count: usize) {
        if self.buffer.len() >= count {
            return;
        }
        for _ in 0..(count - self.buffer.len()) {
            self.buffer.push_back(self.rdr.next().unwrap_or('\0'));
        }
    }
    fn skip(&mut self) {
        let c = self.buffer.pop_front().unwrap();

        self.mark.index += 1;
        if c == '\n' {
            self.mark.line += 1;
            self.mark.col = 0;
        } else {
            self.mark.col += 1;
        }
    }
    fn ch(&self) -> char {
        self.buffer[0]
    }
    fn ch_is(&self, c: char) -> bool {
        self.buffer[0] == c
    }
    #[allow(dead_code)]
    fn eof(&self) -> bool {
        self.ch_is('\0')
    }
    pub fn stream_started(&self) -> bool {
        self.stream_start_produced
    }
    pub fn stream_ended(&self) -> bool {
        self.stream_end_produced
    }
    pub fn mark(&self) -> Marker {
        self.mark
    }
    fn read_break(&mut self, s: &mut String) {
        if self.buffer[0] == '\r' && self.buffer[1] == '\n' {
            s.push('\n');
            self.skip();
            self.skip();
        } else if self.buffer[0] == '\r' || self.buffer[0] == '\n' {
            s.push('\n');
            self.skip();
        } else {
            unreachable!();
        }
    }
    fn insert_token(&mut self, pos: usize, tok: Token) {
        let old_len = self.tokens.len();
        assert!(pos <= old_len);
        self.tokens.push_back(tok);
        for i in 0..old_len - pos {
            self.tokens.swap(old_len - i, old_len - i - 1);
        }
    }
    fn allow_simple_key(&mut self) {
            self.simple_key_allowed = true;
    }
    fn disallow_simple_key(&mut self) {
            self.simple_key_allowed = false;
    }

    pub fn fetch_next_token(&mut self) -> ScanResult {
        self.lookahead(1);
        // println!("--> fetch_next_token Cur {:?} {:?}", self.mark, self.ch());

        if !self.stream_start_produced {
            self.fetch_stream_start();
            return Ok(());
        }
        self.skip_to_next_token();

        try!(self.stale_simple_keys());

        let mark = self.mark;
        self.unroll_indent(mark.col as isize);

        self.lookahead(4);

        if is_z(self.ch()) {
            try!(self.fetch_stream_end());
            return Ok(());
        }

        if self.mark.col == 0 && self.ch_is('%') {
            unimplemented!();
        }

        if self.mark.col == 0
            && self.buffer[0] == '-'
            && self.buffer[1] == '-'
            && self.buffer[2] == '-'
            && is_blankz(self.buffer[3]) {
            try!(self.fetch_document_indicator(TokenType::DocumentStartToken));
            return Ok(());
        }

        if self.mark.col == 0
            && self.buffer[0] == '.'
            && self.buffer[1] == '.'
            && self.buffer[2] == '.'
            && is_blankz(self.buffer[3]) {
            try!(self.fetch_document_indicator(TokenType::DocumentEndToken));
            return Ok(());
        }

        let c = self.buffer[0];
        let nc = self.buffer[1];
        match c {
            '[' => try!(self.fetch_flow_collection_start(TokenType::FlowSequenceStartToken)),
            '{' => try!(self.fetch_flow_collection_start(TokenType::FlowMappingStartToken)),
            ']' => try!(self.fetch_flow_collection_end(TokenType::FlowSequenceEndToken)),
            '}' => try!(self.fetch_flow_collection_end(TokenType::FlowMappingEndToken)),
            ',' => try!(self.fetch_flow_entry()),
            '-' if is_blankz(nc) => try!(self.fetch_block_entry()),
            '?' if self.flow_level > 0 || is_blankz(nc) => unimplemented!(),
            ':' if self.flow_level > 0 || is_blankz(nc) => try!(self.fetch_value()),
            '*' => unimplemented!(),
            '&' => unimplemented!(),
            '!' => unimplemented!(),
            '|' if self.flow_level == 0 => unimplemented!(),
            '>' if self.flow_level == 0 => unimplemented!(),
            '\'' => unimplemented!(),
            '"' => unimplemented!(),
            // plain scalar
            '-' if !is_blankz(nc) => try!(self.fetch_plain_scalar()),
            ':' | '?' if !is_blankz(nc) && self.flow_level == 0 => try!(self.fetch_plain_scalar()),
            '%' | '@' | '`' => return Err(ScanError::new(self.mark,
                    &format!("unexpected character: `{}'", c))),
            _ => try!(self.fetch_plain_scalar()),
        }

        Ok(())
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, ScanError> {
        if self.stream_end_produced {
            return Ok(None);
        }

        if !self.token_available {
            try!(self.fetch_more_tokens());
        }
        let t = self.tokens.pop_front().unwrap();
        self.token_available = false;
        self.tokens_parsed += 1;

        match t.1 {
            TokenType::StreamEndToken => self.stream_end_produced = true,
            _ => {}
        }
        Ok(Some(t))
    }

    pub fn fetch_more_tokens(&mut self) -> ScanResult {
        let mut need_more;
        loop {
            need_more = false;
            if self.tokens.is_empty() {
                need_more = true;
            } else {
                try!(self.stale_simple_keys());
                for sk in &self.simple_keys {
                    if sk.possible && sk.token_number == self.tokens_parsed {
                        need_more = true;
                        break;
                    }
                }
            }

            if !need_more { break; }
            try!(self.fetch_next_token());
        }
        self.token_available = true;

        Ok(())
    }

    fn stale_simple_keys(&mut self) -> ScanResult {
        for sk in &mut self.simple_keys {
            if sk.possible && (sk.mark.line < self.mark.line
                || sk.mark.index + 1024 < self.mark.index) {
                    if sk.required {
                        return Err(ScanError::new(self.mark, "simple key expect ':'"));
                    }
                    sk.possible = false;
                }
        }
        Ok(())
    }

    fn skip_to_next_token(&mut self) {
        loop {
            self.lookahead(1);
            // TODO(chenyh) BOM
            match self.ch() {
                ' ' => self.skip(),
                '\t' if self.flow_level > 0 || !self.simple_key_allowed => self.skip(),
                '\n' | '\r' => {
                    self.skip();
                    if self.flow_level == 0 {
                        self.allow_simple_key();
                    }
                },
                '#' => while !is_breakz(self.ch()) { self.skip(); self.lookahead(1); },
                _ => break
            }
        }
    }

    fn fetch_stream_start(&mut self) {
        let mark = self.mark;
        self.indent = -1;
        self.stream_start_produced = true;
        self.allow_simple_key();
        self.tokens.push_back(Token(mark, TokenType::StreamStartToken(TEncoding::Utf8)));
        self.simple_keys.push(SimpleKey::new(Marker::new(0,0,0)));
    }

    fn fetch_stream_end(&mut self) -> ScanResult {
        // force new line
        if self.mark.col != 0 {
            self.mark.col = 0;
            self.mark.line += 1;
        }

        self.unroll_indent(-1);
        try!(self.remove_simple_key());
        self.disallow_simple_key();

        self.tokens.push_back(Token(self.mark, TokenType::StreamEndToken));
        Ok(())
    }

    fn fetch_flow_collection_start(&mut self, tok :TokenType) -> ScanResult {
        // The indicators '[' and '{' may start a simple key.
        try!(self.save_simple_key());

        self.increase_flow_level();

        self.allow_simple_key();

        let start_mark = self.mark;
        self.skip();

        self.tokens.push_back(Token(start_mark, tok));
        Ok(())
    }

    fn fetch_flow_collection_end(&mut self, tok :TokenType) -> ScanResult {
        try!(self.remove_simple_key());
        self.decrease_flow_level();

        self.disallow_simple_key();

        let start_mark = self.mark;
        self.skip();

        self.tokens.push_back(Token(start_mark, tok));
        Ok(())
    }

    fn fetch_flow_entry(&mut self) -> ScanResult {
        try!(self.remove_simple_key());
        self.allow_simple_key();

        let start_mark = self.mark;
        self.skip();

        self.tokens.push_back(Token(start_mark, TokenType::FlowEntryToken));
        Ok(())
    }

    fn increase_flow_level(&mut self) {
        self.simple_keys.push(SimpleKey::new(Marker::new(0,0,0)));
        self.flow_level += 1;
    }
    fn decrease_flow_level(&mut self) {
        if self.flow_level > 0 {
            self.flow_level -= 1;
            self.simple_keys.pop().unwrap();
        }
    }

    fn fetch_block_entry(&mut self) -> ScanResult {
        if self.flow_level == 0 {
            // Check if we are allowed to start a new entry.
            if !self.simple_key_allowed {
                return Err(ScanError::new(self.mark,
                        "block sequence entries are not allowed in this context"));
            }

            let mark = self.mark;
            // generate BLOCK-SEQUENCE-START if indented
            self.roll_indent(mark.col, None, TokenType::BlockSequenceStartToken, mark);
        } else {
            // - * only allowed in block
            unreachable!();
        }
        try!(self.remove_simple_key());
        self.allow_simple_key();

        let start_mark = self.mark;
        self.skip();

        self.tokens.push_back(Token(start_mark, TokenType::BlockEntryToken));
        Ok(())
    }
    
    fn fetch_document_indicator(&mut self, t: TokenType) -> ScanResult {
        self.unroll_indent(-1);
        try!(self.remove_simple_key());
        self.disallow_simple_key();

        let mark = self.mark;

        self.skip();
        self.skip();
        self.skip();

        self.tokens.push_back(Token(mark, t));
        Ok(())
    }

    fn fetch_plain_scalar(&mut self) -> Result<(), ScanError> {
        try!(self.save_simple_key());

        self.disallow_simple_key();

        let tok = try!(self.scan_plain_scalar());

        self.tokens.push_back(tok);

        Ok(())
    }

    fn scan_plain_scalar(&mut self) -> Result<Token, ScanError> {
        let indent = self.indent + 1;
        let start_mark = self.mark;

        let mut string = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        let mut whitespaces = String::new();
        let mut leading_blanks = false;

        loop {
            /* Check for a document indicator. */
            self.lookahead(4);

            if self.mark.col == 0 &&
                ((self.buffer[0] == '-') &&
                (self.buffer[1] == '-') &&
                (self.buffer[2] == '-')) ||
                ((self.buffer[0] == '.') &&
                (self.buffer[1] == '.') &&
                (self.buffer[2] == '.')) &&
                is_blankz(self.buffer[3]) {
                    break;
                }

            if self.ch() == '#' { break; }
            while !is_blankz(self.ch()) {
                if self.flow_level > 0 && self.ch() == ':'
                    && is_blankz(self.ch()) {
                        return Err(ScanError::new(start_mark,
                            "while scanning a plain scalar, found unexpected ':'"));
                    }
                // indicators ends a plain scalar
                match self.ch() {
                    ':' if is_blankz(self.buffer[1]) => break,
                    ',' | ':' | '?' | '[' | ']' |'{' |'}' if self.flow_level > 0 => break,
                    _ => {}
                }

                if leading_blanks || !whitespaces.is_empty() {
                    if leading_blanks {
                        if !leading_break.is_empty() {
                            if trailing_breaks.is_empty() {
                                string.push(' ');
                            } else {
                                string.extend(trailing_breaks.chars());
                                trailing_breaks.clear();
                            }
                            leading_break.clear();
                        } else {
                            string.extend(leading_break.chars());
                            string.extend(trailing_breaks.chars());
                            trailing_breaks.clear();
                            leading_break.clear();
                        }
                        leading_blanks = false;
                    } else {
                        string.extend(whitespaces.chars());
                        whitespaces.clear();
                    }
                }

                string.push(self.ch());
                self.skip();
                self.lookahead(2);
            }
            // is the end?
            if !(is_blank(self.ch()) || is_break(self.ch())) { break; }
            self.lookahead(1);

            while is_blank(self.ch()) || is_break(self.ch()) {
                if is_blank(self.ch()) {
                    if leading_blanks && (self.mark.col as isize) < indent
                        && self.ch() == '\t' {
                            return Err(ScanError::new(start_mark,
                                "while scanning a plain scalar, found a tab"));
                    }

                    if !leading_blanks {
                        whitespaces.push(self.ch());
                        self.skip();
                    } else {
                        self.skip();
                    }
                } else {
                    self.lookahead(2);
                    // Check if it is a first line break
                    if !leading_blanks {
                        whitespaces.clear();
                        self.read_break(&mut leading_break);
                        leading_blanks = true;
                    } else {
                        self.read_break(&mut trailing_breaks);
                    }
                }
                self.lookahead(1);
            }

            // check intendation level
            if self.flow_level == 0 && (self.mark.col as isize) < indent {
                break;
            }
        }

        if leading_blanks {
            self.allow_simple_key();
        }

        Ok(Token(start_mark, TokenType::ScalarToken(TScalarStyle::Plain, string)))
    }

    fn fetch_value(&mut self) -> ScanResult {
        let sk = self.simple_keys.last().unwrap().clone();
        let start_mark = self.mark;
        if sk.possible {
            let tok = Token(start_mark, TokenType::KeyToken);
            let tokens_parsed = self.tokens_parsed;
            self.insert_token(sk.token_number - tokens_parsed, tok); 

            // Add the BLOCK-MAPPING-START token if needed.
            self.roll_indent(sk.mark.col, Some(sk.token_number),
                TokenType::BlockMappingStartToken, start_mark);

            self.simple_keys.last_mut().unwrap().possible = false;
            self.disallow_simple_key();
        } else {
            // The ':' indicator follows a complex key.
            unimplemented!();
        }

        self.skip();
        self.tokens.push_back(Token(start_mark, TokenType::ValueToken));

        Ok(())
    }

    fn roll_indent(&mut self, col: usize, number: Option<usize>,
                   tok: TokenType, mark: Marker) {
        if self.flow_level > 0 {
            return;
        }

        if self.indent < col as isize {
            self.indents.push(self.indent);
            self.indent = col as isize;
            let tokens_parsed = self.tokens_parsed;
            match number {
                Some(n) => self.insert_token(n - tokens_parsed, Token(mark, tok)),
                None => self.tokens.push_back(Token(mark, tok))
            }
        }
    }

    fn unroll_indent(&mut self, col: isize) {
        if self.flow_level > 0 {
            return;
        }
        while self.indent > col {
            self.tokens.push_back(Token(self.mark, TokenType::BlockEndToken));
            self.indent = self.indents.pop().unwrap();
        }
    }

    fn save_simple_key(&mut self) -> Result<(), ScanError> {
        let required = self.flow_level > 0 && self.indent == (self.mark.col as isize);
        if self.simple_key_allowed {
            let mut sk = SimpleKey::new(self.mark);
            sk.possible = true;
            sk.required = required;
            sk.token_number = self.tokens_parsed + self.tokens.len();

            try!(self.remove_simple_key());

            self.simple_keys.pop();
            self.simple_keys.push(sk);
        }
        Ok(())
    }

    fn remove_simple_key(&mut self) -> ScanResult {
        let last = self.simple_keys.last_mut().unwrap();
        if last.possible {
            if last.required {
                return Err(ScanError::new(self.mark, "simple key expected"));
            }
        }

        last.possible = false;
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tokenizer() {
        let s: String = "---
# comment
a0 bb: val
a1:
    b1: 4
    b2: d
a2: 4
a3: [1, 2, 3]
a4:
    - - a1
      - a2
    - 2
".to_string();
        let p = Scanner::new(s.chars());
        for t in p {
            println!("{:?}", t);
        }
    }
}

