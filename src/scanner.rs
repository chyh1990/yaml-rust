use std::collections::VecDeque;
use std::char;

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
    /// major, minor
    VersionDirectiveToken(u32, u32),
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
    error: Option<ScanError>,

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
        if self.error.is_some() {
            return None;
        }
        match self.next_token() {
            Ok(tok) => tok,
            Err(e) => {
                self.error = Some(e);
                None
            }
        }
    }
}

#[inline]
fn is_z(c: char) -> bool {
    c == '\0'
}
#[inline]
fn is_break(c: char) -> bool {
    c == '\n' || c == '\r'
}
#[inline]
fn is_breakz(c: char) -> bool {
    is_break(c) || is_z(c)
}
#[inline]
fn is_blank(c: char) -> bool {
    c == ' ' || c == '\t'
}
#[inline]
fn is_blankz(c: char) -> bool {
    is_blank(c) || is_breakz(c)
}
#[inline]
fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}
#[inline]
fn is_hex(c: char) -> bool {
    (c >= '0' && c <= '9')
        || (c >= 'a' && c <= 'f')
        || (c >= 'A' && c <= 'F')
}
#[inline]
fn as_hex(c: char) -> u32 {
    match c {
        '0'...'9' => (c as u32) - ('0' as u32),
        'a'...'f' => (c as u32) - ('a' as u32) + 10,
        'A'...'F' => (c as u32) - ('A' as u32) + 10,
        _ => unreachable!()
    }
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
            error: None,

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
    #[inline]
    pub fn get_error(&self) -> Option<ScanError> {
        match self.error {
            None => None,
            Some(ref e) => Some(e.clone()),
        }
    }

    #[inline]
    fn lookahead(&mut self, count: usize) {
        if self.buffer.len() >= count {
            return;
        }
        for _ in 0..(count - self.buffer.len()) {
            self.buffer.push_back(self.rdr.next().unwrap_or('\0'));
        }
    }
    #[inline]
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
    #[inline]
    fn skip_line(&mut self) {
        if self.buffer[0] == '\r' && self.buffer[1] == '\n' {
            self.skip();
            self.skip();
        } else if is_break(self.buffer[0]) {
            self.skip();
        }
    }
    #[inline]
    fn ch(&self) -> char {
        self.buffer[0]
    }
    #[inline]
    fn ch_is(&self, c: char) -> bool {
        self.buffer[0] == c
    }
    #[allow(dead_code)]
    #[inline]
    fn eof(&self) -> bool {
        self.ch_is('\0')
    }
    #[inline]
    pub fn stream_started(&self) -> bool {
        self.stream_start_produced
    }
    #[inline]
    pub fn stream_ended(&self) -> bool {
        self.stream_end_produced
    }
    #[inline]
    pub fn mark(&self) -> Marker {
        self.mark
    }
    #[inline]
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

        // Is it a directive?
        if self.mark.col == 0 && self.ch_is('%') {
            return self.fetch_directive();
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
            '?' if self.flow_level > 0 || is_blankz(nc) => try!(self.fetch_key()),
            ':' if self.flow_level > 0 || is_blankz(nc) => try!(self.fetch_value()),
            '*' => unimplemented!(),
            '&' => unimplemented!(),
            '!' => unimplemented!(),
            // Is it a literal scalar?
            '|' if self.flow_level == 0 => try!(self.fetch_block_scalar(true)),
            // Is it a folded scalar?
            '>' if self.flow_level == 0 => try!(self.fetch_block_scalar(false)),
            '\'' => try!(self.fetch_flow_scalar(true)),
            '"' => try!(self.fetch_flow_scalar(false)),
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
                    self.lookahead(2);
                    self.skip_line();
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

    fn fetch_directive(&mut self) -> ScanResult {
        self.unroll_indent(-1);
        try!(self.remove_simple_key());

        self.disallow_simple_key();

        let tok = try!(self.scan_directive());

        self.tokens.push_back(tok);

        Ok(())
    }

    fn scan_directive(&mut self) -> Result<Token, ScanError> {
        let start_mark = self.mark;
        self.skip();

        let name = try!(self.scan_directive_name());
        let tok = match name.as_ref() {
            "YAML" => {
                try!(self.scan_version_directive_value(&start_mark))
            },
            "TAG" => {
                try!(self.scan_tag_directive_value(&start_mark))
            },
            _ => return Err(ScanError::new(start_mark,
                "while scanning a directive, found uknown directive name"))
        };
        self.lookahead(1);

        while is_blank(self.ch()) {
            self.skip();
            self.lookahead(1);
        }

        if self.ch() == '#' {
            while !is_breakz(self.ch()) {
                self.skip();
                self.lookahead(1);
            }
        }

        if !is_breakz(self.ch()) {
            return Err(ScanError::new(start_mark,
                "while scanning a directive, did not find expected comment or line break"));
        }

        // Eat a line break
        if is_break(self.ch()) {
            self.lookahead(2);
            self.skip_line();
        }

        Ok(tok)
    }

    fn scan_version_directive_value(&mut self, mark: &Marker) -> Result<Token, ScanError> {
        self.lookahead(1);

        while is_blank(self.ch()) {
            self.skip();
            self.lookahead(1);
        }

        let major = try!(self.scan_version_directive_number(mark));

        if self.ch() != '.' {
            return Err(ScanError::new(*mark,
                "while scanning a YAML directive, did not find expected digit or '.' character"));
        }

        self.skip();

        let minor = try!(self.scan_version_directive_number(mark));

        Ok(Token(*mark, TokenType::VersionDirectiveToken(major, minor)))
    }

    fn scan_directive_name(&mut self) -> Result<String, ScanError> {
        let start_mark = self.mark;
        let mut string = String::new();
        self.lookahead(1);
        while self.ch().is_alphabetic() {
            string.push(self.ch());
            self.skip();
            self.lookahead(1);
        }

        if string.is_empty() {
            return Err(ScanError::new(start_mark, 
                    "while scanning a directive, could not find expected directive name"));
        }

        if !is_blankz(self.ch()) {
            return Err(ScanError::new(start_mark, 
                    "while scanning a directive, found unexpected non-alphabetical character"));
        }

        Ok(string)
    }

    fn scan_version_directive_number(&mut self, mark: &Marker) -> Result<u32, ScanError> {
        let mut val = 0u32;
        let mut length = 0usize;
        self.lookahead(1);
        while is_digit(self.ch()) {
            if length + 1 > 9 {
                return Err(ScanError::new(*mark,
                    "while scanning a YAML directive, found extremely long version number"));
            }
            length += 1;
            val = val * 10 + ((self.ch() as u32) - ('0' as u32));
            self.skip();
            self.lookahead(1);
        }

        if length == 0 {
                return Err(ScanError::new(*mark,
                    "while scanning a YAML directive, did not find expected version number"));
        }

        Ok(val)
    }

    fn scan_tag_directive_value(&mut self, mark: &Marker) -> Result<Token, ScanError> {
        unimplemented!();
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

    fn fetch_block_scalar(&mut self, literal: bool) -> ScanResult {
        try!(self.save_simple_key());
        self.allow_simple_key();
        let tok = try!(self.scan_block_scalar(literal));

        self.tokens.push_back(tok);
        Ok(())
    }

    fn scan_block_scalar(&mut self, literal: bool) -> Result<Token, ScanError> {
        let start_mark = self.mark;
        let mut chomping: i32 = 0;
        let mut increment: usize = 0;
        let mut indent: usize = 0;
        let mut trailing_blank: bool;
        let mut leading_blank: bool = false;

        let mut string = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();

        // skip '|' or '>'
        self.skip();
        self.lookahead(1);

        if self.ch() == '+' || self.ch() == '-' {
            if self.ch() == '+' {
                chomping = 1;
            } else {
                chomping = -1;
            }
            self.skip();
            self.lookahead(1);
            if is_digit(self.ch()) {
                if self.ch() == '0' {
                    return Err(ScanError::new(start_mark,
                            "while scanning a block scalar, found an intendation indicator equal to 0"));
                }
                increment = (self.ch() as usize) - ('0' as usize);
                self.skip();
            }
        } else if is_digit(self.ch()) {
            if self.ch() == '0' {
                return Err(ScanError::new(start_mark,
                         "while scanning a block scalar, found an intendation indicator equal to 0"));
            }

            increment = (self.ch() as usize) - ('0' as usize);
            self.skip();
            self.lookahead(1);
            if self.ch() == '+' || self.ch() == '-' {
                if self.ch() == '+' {
                    chomping = 1;
                } else {
                    chomping = -1;
                }
                self.skip();
            }
        }

        // Eat whitespaces and comments to the end of the line.
        self.lookahead(1);

        while is_blank(self.ch()) {
            self.skip();
            self.lookahead(1);
        }

        if self.ch() == '#' {
            while !is_breakz(self.ch()) {
                self.skip();
                self.lookahead(1);
            }
        }

        // Check if we are at the end of the line.
        if !is_breakz(self.ch()) {
            return Err(ScanError::new(start_mark,
                    "while scanning a block scalar, did not find expected comment or line break"));
        }

        if is_break(self.ch()) {
            self.lookahead(2);
            self.skip_line();
        }

        if increment > 0 {
            indent = if self.indent >= 0 { (self.indent + increment as isize) as usize } else { increment }
        }
        // Scan the leading line breaks and determine the indentation level if needed.
        try!(self.block_scalar_breaks(&mut indent, &mut trailing_breaks));
        
        self.lookahead(1);

        let start_mark = self.mark;

        while self.mark.col == indent && !is_z(self.ch()) {
            // We are at the beginning of a non-empty line.
            trailing_blank = is_blank(self.ch());
            if !literal && !leading_break.is_empty()
                && !leading_blank && !trailing_blank {
                    if trailing_breaks.is_empty() {
                        string.push(' ');
                    }
                    leading_break.clear();
            } else {
                string.extend(leading_break.chars());
                leading_break.clear();
            }

            string.extend(trailing_breaks.chars());
            trailing_breaks.clear();

            leading_blank = is_blank(self.ch());

            while !is_breakz(self.ch()) {
                string.push(self.ch());
                self.skip();
                self.lookahead(1);
            }

            self.lookahead(2);
            self.skip_line();

            // Eat the following intendation spaces and line breaks.
            try!(self.block_scalar_breaks(&mut indent, &mut trailing_breaks));
        }

        // Chomp the tail.
        if chomping != -1 {
            string.extend(leading_break.chars());
        }

        if chomping == 1 {
            string.extend(trailing_breaks.chars());
        }

        if literal {
            Ok(Token(start_mark, TokenType::ScalarToken(TScalarStyle::Literal, string)))
        } else {
            Ok(Token(start_mark, TokenType::ScalarToken(TScalarStyle::Foled, string)))
        }
    }

    fn block_scalar_breaks(&mut self, indent: &mut usize, breaks: &mut String) -> ScanResult {
        let mut max_indent = 0;
        loop {
            self.lookahead(1);
            while (*indent == 0 || self.mark.col < *indent)
                && self.buffer[0] == ' ' {
                    self.skip();
                    self.lookahead(1);
            }

            if self.mark.col > max_indent {
                max_indent = self.mark.col;
            }

            // Check for a tab character messing the intendation.
            if (*indent == 0 || self.mark.col < *indent)
                && self.buffer[0] == '\t' {
                return Err(ScanError::new(self.mark, 
                        "while scanning a block scalar, found a tab character where an intendation space is expected"));
            }

            if !is_break(self.ch()) {
                break;
            }

            self.lookahead(2);
            // Consume the line break.
            self.read_break(breaks);
        }

        if *indent == 0 {
            *indent = max_indent;
            if *indent < (self.indent + 1) as usize {
                *indent = (self.indent + 1) as usize;
            }
            if *indent < 1 {
                *indent = 1;
            }
        }
        Ok(())
    }

    fn fetch_flow_scalar(&mut self, single: bool) -> ScanResult {
        try!(self.save_simple_key());
        self.disallow_simple_key();

        let tok = try!(self.scan_flow_scalar(single));

        self.tokens.push_back(tok);
        Ok(())
    }

    fn scan_flow_scalar(&mut self, single: bool) -> Result<Token, ScanError> {
        let start_mark = self.mark;

        let mut string = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        let mut whitespaces = String::new();
        let mut leading_blanks;

        /* Eat the left quote. */
        self.skip();

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
                    return Err(ScanError::new(start_mark, 
                        "while scanning a quoted scalar, found unexpected document indicator"));
                }

            if is_z(self.ch()) {
                    return Err(ScanError::new(start_mark, 
                        "while scanning a quoted scalar, found unexpected end of stream"));
            }

            self.lookahead(2);

            leading_blanks = false;
            // Consume non-blank characters.

            while !is_blankz(self.ch()) {
                match self.ch() {
                    // Check for an escaped single quote.
                    '\'' if self.buffer[1] == '\'' && single => {
                        string.push('\'');
                        self.skip();
                        self.skip();
                    },
                    // Check for the right quote.
                    '\'' if single => { break; },
                    '"' if !single => { break; },
                    // Check for an escaped line break.
                    '\\' if !single && is_break(self.buffer[1]) => {
                        self.lookahead(3);
                        self.skip();
                        self.skip_line();
                        leading_blanks = true;
                        break;
                    }
                    // Check for an escape sequence.
                    '\\' if !single => {
                        let mut code_length = 0usize;
                        match self.buffer[1] {
                            '0' => string.push('\0'),
                            'a' => string.push('\x07'),
                            'b' => string.push('\x08'),
                            't' | '\t' => string.push('\t'),
                            'n' => string.push('\n'),
                            'v' => string.push('\x0b'),
                            'f' => string.push('\x0c'),
                            'r' => string.push('\x0d'),
                            'e' => string.push('\x1b'),
                            ' ' => string.push('\x20'),
                            '"' => string.push('"'),
                            '\'' => string.push('\''),
                            '\\' => string.push('\\'),
                            // NEL (#x85)
                            'N' => string.push(char::from_u32(0x85).unwrap()),
                            // #xA0
                            '_' => string.push(char::from_u32(0xA0).unwrap()),
                            // LS (#x2028)
                            'L' => string.push(char::from_u32(0x2028).unwrap()),
                            // PS (#x2029)
                            'P' => string.push(char::from_u32(0x2029).unwrap()),
                            'x' => code_length = 2,
                            'u' => code_length = 4,
                            'U' => code_length = 8,
                            _ => return Err(ScanError::new(start_mark,
                                    "while parsing a quoted scalar, found unknown escape character"))
                        }
                        self.skip();
                        self.skip();
                        // Consume an arbitrary escape code.
                        if code_length > 0 {
                            self.lookahead(code_length);
                            let mut value = 0u32;
                            for i in 0..code_length {
                                if !is_hex(self.buffer[i]) {
                                    return Err(ScanError::new(start_mark,
                                        "while parsing a quoted scalar, did not find expected hexdecimal number"));
                                }
                                value = (value << 4) + as_hex(self.buffer[i]);
                            }

                            let ch = match char::from_u32(value) {
                                Some(v) => v,
                                None => {
                                    return Err(ScanError::new(start_mark,
                                        "while parsing a quoted scalar, found invalid Unicode character escape code"));
                                }
                            };
                            string.push(ch);

                            for _ in 0..code_length {
                                self.skip();
                            }
                        }
                    },
                    c => { string.push(c); self.skip(); }
                }
                self.lookahead(2);
            }
            match self.ch() {
                '\'' if single => { break; },
                '"' if !single => { break; },
                _ => {}
            }
            self.lookahead(1);

            // Consume blank characters.
            while is_blank(self.ch()) || is_break(self.ch()) {
                if is_blank(self.ch()) {
                    // Consume a space or a tab character.
                    if !leading_blanks {
                        whitespaces.push(self.ch());
                        self.skip();
                    } else {
                        self.skip();
                    }
                } else {
                    self.lookahead(2);
                    // Check if it is a first line break.
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
            // Join the whitespaces or fold line breaks.
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
        } // loop

        // Eat the right quote.
        self.skip();

        if single {
            Ok(Token(start_mark, TokenType::ScalarToken(TScalarStyle::SingleQuoted, string)))
        } else {
            Ok(Token(start_mark, TokenType::ScalarToken(TScalarStyle::DoubleQuoted, string)))
        }
    }

    fn fetch_plain_scalar(&mut self) -> ScanResult {
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

    fn fetch_key(&mut self) -> ScanResult {
        let start_mark = self.mark;
        if self.flow_level == 0 {
            // Check if we are allowed to start a new key (not nessesary simple).
            if !self.simple_key_allowed {
                return Err(ScanError::new(self.mark, "mapping keys are not allowed in this context"));
            }
            self.roll_indent(start_mark.col, None,
                TokenType::BlockMappingStartToken, start_mark);
        }

        try!(self.remove_simple_key());

        if self.flow_level == 0 {
            self.allow_simple_key();
        } else {
            self.disallow_simple_key();
        }

        self.skip();
        self.tokens.push_back(Token(start_mark, TokenType::KeyToken));
        Ok(())
    }

    fn fetch_value(&mut self) -> ScanResult {
        let sk = self.simple_keys.last().unwrap().clone();
        let start_mark = self.mark;
        if sk.possible {
            // insert simple key
            let tok = Token(sk.mark, TokenType::KeyToken);
            let tokens_parsed = self.tokens_parsed;
            self.insert_token(sk.token_number - tokens_parsed, tok); 

            // Add the BLOCK-MAPPING-START token if needed.
            self.roll_indent(sk.mark.col, Some(sk.token_number),
                TokenType::BlockMappingStartToken, start_mark);

            self.simple_keys.last_mut().unwrap().possible = false;
            self.disallow_simple_key();
        } else {
            // The ':' indicator follows a complex key.
            if self.flow_level == 0 {
                if !self.simple_key_allowed {
                    return Err(ScanError::new(start_mark,
                        "mapping values are not allowed in this context"));
                }

                self.roll_indent(start_mark.col, None,
                    TokenType::BlockMappingStartToken, start_mark);
            }

            if self.flow_level == 0 {
                self.allow_simple_key();
            } else {
                self.disallow_simple_key();
            }
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
    use super::TokenType::*;

macro_rules! next {
    ($p:ident, $tk:pat) => {{
        let tok = $p.next().unwrap();
        match tok.1 {
            $tk => {},
            _ => { panic!("unexpected token: {:?}",
                    tok) }
        }
    }}
}

macro_rules! next_scalar {
    ($p:ident, $tk:expr, $v:expr) => {{
        let tok = $p.next().unwrap();
        match tok.1 {
            ScalarToken(style, ref v) => {
                assert_eq!(style, $tk);
                assert_eq!(v, $v);
            },
            _ => { panic!("unexpected token: {:?}",
                    tok) }
        }
    }}
}

macro_rules! end {
    ($p:ident) => {{
        assert_eq!($p.next(), None);
    }}
}
    /// test cases in libyaml scanner.c
    #[test]
    fn test_empty() {
        let s = "";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_scalar() {
        let s = "a scalar";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_explicit_scalar() {
        let s = 
"---
'a scalar'
...
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, DocumentStartToken);
        next!(p, ScalarToken(TScalarStyle::SingleQuoted, _));
        next!(p, DocumentEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_multiple_documents() {
        let s = 
"
'a scalar'
---
'a scalar'
---
'a scalar'
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, ScalarToken(TScalarStyle::SingleQuoted, _));
        next!(p, DocumentStartToken);
        next!(p, ScalarToken(TScalarStyle::SingleQuoted, _));
        next!(p, DocumentStartToken);
        next!(p, ScalarToken(TScalarStyle::SingleQuoted, _));
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_a_flow_sequence() {
        let s = "[item 1, item 2, item 3]";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, FlowSequenceStartToken);
        next_scalar!(p, TScalarStyle::Plain, "item 1");
        next!(p, FlowEntryToken);
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, FlowEntryToken);
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, FlowSequenceEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_a_flow_mapping() {
        let s = 
"
{
    a simple key: a value, # Note that the KEY token is produced.
    ? a complex key: another value,
}
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, FlowMappingStartToken);
        next!(p, KeyToken);
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, ValueToken);
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, FlowEntryToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "a complex key");
        next!(p, ValueToken);
        next!(p, ScalarToken(TScalarStyle::Plain, _));
        next!(p, FlowEntryToken);
        next!(p, FlowMappingEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_block_sequences() {
        let s = 
"
- item 1
- item 2
-
  - item 3.1
  - item 3.2
-
  key 1: value 1
  key 2: value 2
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 1");
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 2");
        next!(p, BlockEntryToken);
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 3.1");
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 3.2");
        next!(p, BlockEndToken);
        next!(p, BlockEntryToken);
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 1");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 1");
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 2");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 2");
        next!(p, BlockEndToken);
        next!(p, BlockEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_block_mappings() {
        let s = 
"
a simple key: a value   # The KEY token is produced here.
? a complex key
: another value
a mapping:
  key 1: value 1
  key 2: value 2
a sequence:
  - item 1
  - item 2
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken);
        next!(p, ScalarToken(_, _));
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken);
        next!(p, ScalarToken(_, _));
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken); // libyaml comment seems to be wrong
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken);
        next!(p, ScalarToken(_, _));
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken);
        next!(p, ScalarToken(_, _));
        next!(p, BlockEndToken);
        next!(p, KeyToken);
        next!(p, ScalarToken(_, _));
        next!(p, ValueToken);
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next!(p, ScalarToken(_, _));
        next!(p, BlockEntryToken);
        next!(p, ScalarToken(_, _));
        next!(p, BlockEndToken);
        next!(p, BlockEndToken);
        next!(p, StreamEndToken);
        end!(p);

    }

    #[test]
    fn test_no_block_sequence_start() {
        let s =
"
key:
- item 1
- item 2
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key");
        next!(p, ValueToken);
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 1");
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 2");
        next!(p, BlockEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_collections_in_sequence() {
        let s =
"
- - item 1
  - item 2
- key 1: value 1
  key 2: value 2
- ? complex key
  : complex value
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 1");
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 2");
        next!(p, BlockEndToken);
        next!(p, BlockEntryToken);
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 1");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 1");
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 2");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 2");
        next!(p, BlockEndToken);
        next!(p, BlockEntryToken);
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "complex key");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "complex value");
        next!(p, BlockEndToken);
        next!(p, BlockEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_collections_in_mapping() {
        let s =
"
? a sequence
: - item 1
  - item 2
? a mapping
: key 1: value 1
  key 2: value 2
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "a sequence");
        next!(p, ValueToken);
        next!(p, BlockSequenceStartToken);
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 1");
        next!(p, BlockEntryToken);
        next_scalar!(p, TScalarStyle::Plain, "item 2");
        next!(p, BlockEndToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "a mapping");
        next!(p, ValueToken);
        next!(p, BlockMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 1");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 1");
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "key 2");
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "value 2");
        next!(p, BlockEndToken);
        next!(p, BlockEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }

    #[test]
    fn test_spec_ex7_3() {
        let s =
"
{
    ? foo :,
    : bar,
}
";
        let mut p = Scanner::new(s.chars());
        next!(p, StreamStartToken(..));
        next!(p, FlowMappingStartToken);
        next!(p, KeyToken);
        next_scalar!(p, TScalarStyle::Plain, "foo");
        next!(p, ValueToken);
        next!(p, FlowEntryToken);
        next!(p, ValueToken);
        next_scalar!(p, TScalarStyle::Plain, "bar");
        next!(p, FlowEntryToken);
        next!(p, FlowMappingEndToken);
        next!(p, StreamEndToken);
        end!(p);
    }
}

