#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::collections::VecDeque;
use std::error::Error;
use std::{char, fmt};

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum TEncoding {
    Utf8,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum TScalarStyle {
    Any,
    Plain,
    SingleQuoted,
    DoubleQuoted,

    Literal,
    Foled,
}

/// A location in a yaml document.
#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub struct Marker {
    index: usize,
    line: usize,
    col: usize,
}

impl Marker {
    fn new(index: usize, line: usize, col: usize) -> Marker {
        Marker { index, line, col }
    }

    /// Return the index (in bytes) of the marker in the source.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the line of the marker in the source.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Return the column of the marker in the source.
    #[must_use]
    pub fn col(&self) -> usize {
        self.col
    }
}

/// An error that occured while scanning.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ScanError {
    mark: Marker,
    info: String,
}

impl ScanError {
    /// Create a new error from a location and an error string.
    #[must_use]
    pub fn new(loc: Marker, info: &str) -> ScanError {
        ScanError {
            mark: loc,
            info: info.to_owned(),
        }
    }

    /// Return the marker pointing to the error in the source.
    #[must_use]
    pub fn marker(&self) -> &Marker {
        &self.mark
    }

    /// Return the information string describing the error that happened.
    #[must_use]
    pub fn info(&self) -> &str {
        self.info.as_ref()
    }
}

impl Error for ScanError {
    fn description(&self) -> &str {
        self.info.as_ref()
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl fmt::Display for ScanError {
    // col starts from 0
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} at line {} column {}",
            self.info,
            self.mark.line,
            self.mark.col + 1
        )
    }
}

/// The contents of a scanner token.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum TokenType {
    NoToken,
    /// The start of the stream. Sent first, before even [`DocumentStart`].
    StreamStart(TEncoding),
    /// The end of the stream, EOF.
    StreamEnd,
    VersionDirective(
        /// Major
        u32,
        /// Minor
        u32,
    ),
    TagDirective(
        /// Handle
        String,
        /// Prefix
        String,
    ),
    /// The start of a YAML document (`---`).
    DocumentStart,
    /// The end of a YAML document (`...`).
    DocumentEnd,
    /// The start of a sequence block.
    ///
    /// Sequence blocks are arrays starting with a `-`.
    BlockSequenceStart,
    /// The start of a sequence mapping.
    ///
    /// Sequence mappings are "dictionaries" with "key: value" entries.
    BlockMappingStart,
    /// End of the corresponding `BlockSequenceStart` or `BlockMappingStart`.
    BlockEnd,
    /// Start of an inline array (`[ a, b ]`).
    FlowSequenceStart,
    /// End of an inline array.
    FlowSequenceEnd,
    /// Start of an inline mapping (`{ a: b, c: d }`).
    FlowMappingStart,
    /// End of an inline mapping.
    FlowMappingEnd,
    /// An entry in a block sequence (c.f.: [`TokenType::BlockSequenceStart`]).
    BlockEntry,
    /// An entry in a flow sequence (c.f.: [`TokenType::FlowSequenceStart`]).
    FlowEntry,
    /// A key in a mapping.
    Key,
    /// A value in a mapping.
    Value,
    /// A reference to an anchor.
    Alias(String),
    /// A YAML anchor (`&`/`*`).
    Anchor(String),
    /// A YAML tag (starting with bangs `!`).
    Tag(
        /// The handle of the tag.
        String,
        /// The suffix of the tag.
        String,
    ),
    /// A regular YAML scalar.
    Scalar(TScalarStyle, String),
}

/// A scanner token.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Token(pub Marker, pub TokenType);

/// A scalar that was parsed and may correspond to a simple key.
///
/// Upon scanning the following yaml:
/// ```yaml
/// a: b
/// ```
/// We do not know that `a` is a key for a map until we have reached the following `:`. For this
/// YAML, we would store `a` as a scalar token in the [`Scanner`], but not emit it yet. It would be
/// kept inside the scanner until more context is fetched and we are able to know whether it is a
/// plain scalar or a key.
///
/// For example, see the following 2 yaml documents:
/// ```yaml
/// ---
/// a: b # Here, `a` is a key.
/// ...
/// ---
/// a # Here, `a` is a plain scalar.
/// ...
/// ```
/// An instance of [`SimpleKey`] is created in the [`Scanner`] when such ambiguity occurs.
///
/// In both documents, scanning `a` would lead to the creation of a [`SimpleKey`] with
/// [`Self::possible`] set to `true`. The token for `a` would be pushed in the [`Scanner`] but not
/// yet emitted. Instead, more context would be fetched (through [`Scanner::fetch_more_tokens`]).
///
/// In the first document, upon reaching the `:`, the [`SimpleKey`] would be inspected and our
/// scalar `a` since it is a possible key, would be "turned" into a key. This is done by prepending
/// a [`TokenType::Key`] to our scalar token in the [`Scanner`]. This way, the
/// [`crate::parser::Parser`] would read the [`TokenType::Key`] token before the
/// [`TokenType::Scalar`] token.
///
/// In the second document however, reaching the EOF would stale the [`SimpleKey`] and no
/// [`TokenType::Key`] would be emitted by the scanner.
#[derive(Clone, PartialEq, Debug, Eq)]
struct SimpleKey {
    /// Whether the token this [`SimpleKey`] refers to may still be a key.
    ///
    /// Sometimes, when we have more context, we notice that what we thought could be a key no
    /// longer can be. In that case, [`Self::possible`] is set to `false`.
    ///
    /// For instance, let us consider the following invalid YAML:
    /// ```yaml
    /// key
    ///   : value
    /// ```
    /// Upon reading the `\n` after `key`, the [`SimpleKey`] that was created for `key` is staled
    /// and [`Self::possible`] set to `false`.
    possible: bool,
    /// Whether the token this [`SimpleKey`] refers to is required to be a key.
    ///
    /// With more context, we may know for sure that the token must be a key. If the YAML is
    /// invalid, it may happen that the token be deemed not a key. In such event, an error has to
    /// be raised. This boolean helps us know when to raise such error.
    ///
    /// TODO(ethiraric, 30/12/2023): Example of when this happens.
    required: bool,
    /// The index of the token referred to by the [`SimpleKey`].
    ///
    /// This is the index in the scanner, which takes into account both the tokens that have been
    /// emitted and those about to be emitted. See [`Scanner::tokens_parsed`] and
    /// [`Scanner::tokens`] for more details.
    token_number: usize,
    /// The position at which the token the [`SimpleKey`] refers to is.
    mark: Marker,
}

impl SimpleKey {
    /// Create a new [`SimpleKey`] at the given `Marker` and with the given flow level.
    fn new(mark: Marker) -> SimpleKey {
        SimpleKey {
            possible: false,
            required: false,
            token_number: 0,
            mark,
        }
    }
}

/// An indentation level on the stack of indentations.
#[derive(Clone, Debug, Default)]
struct Indent {
    /// The former indentation level.
    indent: isize,
    /// Whether, upon closing, this indents generates a `BlockEnd` token.
    ///
    /// There are levels of indentation which do not start a block. Examples of this would be:
    /// ```yaml
    /// -
    ///   foo # ok
    /// -
    /// bar # ko, bar needs to be indented further than the `-`.
    /// - [
    ///  baz, # ok
    /// quux # ko, quux needs to be indented further than the '-'.
    /// ] # ko, the closing bracket needs to be indented further than the `-`.
    /// ```
    ///
    /// The indentation level created by the `-` is for a single entry in the sequence. Emitting a
    /// `BlockEnd` when this indentation block ends would generate one `BlockEnd` per entry in the
    /// sequence, although we must have exactly one to end the sequence.
    needs_block_end: bool,
}

/// The YAML scanner.
///
/// This corresponds to the low-level interface when reading YAML. The scanner emits token as they
/// are read (akin to a lexer), but it also holds sufficient context to be able to disambiguate
/// some of the constructs. It has understanding of indentation and whitespace and is able to
/// generate error messages for some invalid YAML constructs.
///
/// It is however not a full parser and needs [`parser::Parser`] to fully detect invalid YAML
/// documents.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Scanner<T> {
    /// The reader, providing with characters.
    rdr: T,
    /// The position of the cursor within the reader.
    mark: Marker,
    /// Buffer for tokens to be returned.
    ///
    /// This buffer can hold some temporary tokens that are not yet ready to be returned. For
    /// instance, if we just read a scalar, it can be a value or a key if an implicit mapping
    /// follows. In this case, the token stays in the `VecDeque` but cannot be returned from
    /// [`Self::next`] until we have more context.
    tokens: VecDeque<Token>,
    /// Buffer for the next characters to consume.
    buffer: VecDeque<char>,
    /// The last error that happened.
    error: Option<ScanError>,

    /// Whether we have already emitted the `StreamStart` token.
    stream_start_produced: bool,
    /// Whether we have already emitted the `StreamEnd` token.
    stream_end_produced: bool,
    adjacent_value_allowed_at: usize,
    /// Whether a simple key could potentially start at the current position.
    ///
    /// Simple keys are the opposite of complex keys which are keys starting with `?`.
    simple_key_allowed: bool,
    /// A stack of potential simple keys.
    ///
    /// Refer to the documentation of [`SimpleKey`] for a more in-depth explanation of what they
    /// are.
    simple_keys: Vec<SimpleKey>,
    /// The current indentation level.
    indent: isize,
    /// List of all block indentation levels we are in (except the current one).
    indents: Vec<Indent>,
    /// Level of nesting of flow sequences.
    flow_level: u8,
    /// The number of tokens that have been returned from the scanner.
    ///
    /// This excludes the tokens from [`Self::tokens`].
    tokens_parsed: usize,
    /// Whether a token is ready to be taken from [`Self::tokens`].
    token_available: bool,
    /// Whether all characters encountered since the last newline were whitespace.
    leading_whitespace: bool,
    /// Whether we started a flow mapping.
    ///
    /// This is used to detect implicit flow mapping starts such as:
    /// ```yaml
    /// [ : foo ] # { null: "foo" }
    /// ```
    flow_mapping_started: bool,
    /// Whether we currently are in an implicit flow mapping.
    implicit_flow_mapping: bool,
}

impl<T: Iterator<Item = char>> Iterator for Scanner<T> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        if self.error.is_some() {
            return None;
        }
        match self.next_token() {
            Ok(Some(tok)) => {
                if std::env::var("YAMLRUST_DEBUG").is_ok() {
                    eprintln!(
                        "    \x1B[;32m\u{21B3} {:?} \x1B[;36m{:?}\x1B[;m",
                        tok.1, tok.0
                    );
                }
                Some(tok)
            }
            Ok(tok) => tok,
            Err(e) => {
                self.error = Some(e);
                None
            }
        }
    }
}

/// Check whether the character is nil (`\0`).
#[inline]
fn is_z(c: char) -> bool {
    c == '\0'
}

/// Check whether the character is a line break (`\r` or `\n`).
#[inline]
fn is_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

/// Check whether the character is nil or a line break (`\0`, `\r`, `\n`).
#[inline]
fn is_breakz(c: char) -> bool {
    is_break(c) || is_z(c)
}

/// Check whether the character is a whitespace (` ` or `\t`).
#[inline]
fn is_blank(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Check whether the character is nil, a linebreak or a whitespace.
///
/// `\0`, ` `, `\t`, `\n`, `\r`
#[inline]
fn is_blankz(c: char) -> bool {
    is_blank(c) || is_breakz(c)
}

/// Check whether the character is an ascii digit.
#[inline]
fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Check whether the character is a digit, letter, `_` or `-`.
#[inline]
fn is_alpha(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '-')
}

/// Check whether the character is a hexadecimal character (case insensitive).
#[inline]
fn is_hex(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

/// Convert the hexadecimal digit to an integer.
#[inline]
fn as_hex(c: char) -> u32 {
    match c {
        '0'..='9' => (c as u32) - ('0' as u32),
        'a'..='f' => (c as u32) - ('a' as u32) + 10,
        'A'..='F' => (c as u32) - ('A' as u32) + 10,
        _ => unreachable!(),
    }
}

/// Check whether the character is a YAML flow character (one of `,[]{}`).
#[inline]
fn is_flow(c: char) -> bool {
    matches!(c, ',' | '[' | ']' | '{' | '}')
}

/// Check whether the character is the BOM character.
#[inline]
fn is_bom(c: char) -> bool {
    c == '\u{FEFF}'
}

/// Check whether the character is a YAML non-breaking character.
#[inline]
fn is_yaml_non_break(c: char) -> bool {
    // TODO(ethiraric, 28/12/2023): is_printable
    !is_break(c) && !is_bom(c)
}

/// Check whether the character is NOT a YAML whitespace (` ` / `\t`).
#[inline]
fn is_yaml_non_space(c: char) -> bool {
    is_yaml_non_break(c) && !is_blank(c)
}

/// Check whether the character is a valid YAML anchor name character.
#[inline]
fn is_anchor_char(c: char) -> bool {
    is_yaml_non_space(c) && !is_flow(c) && !is_z(c)
}

pub type ScanResult = Result<(), ScanError>;

impl<T: Iterator<Item = char>> Scanner<T> {
    /// Creates the YAML tokenizer.
    pub fn new(rdr: T) -> Scanner<T> {
        Scanner {
            rdr,
            buffer: VecDeque::new(),
            mark: Marker::new(0, 1, 0),
            tokens: VecDeque::new(),
            error: None,

            stream_start_produced: false,
            stream_end_produced: false,
            adjacent_value_allowed_at: 0,
            simple_key_allowed: true,
            simple_keys: Vec::new(),
            indent: -1,
            indents: Vec::new(),
            flow_level: 0,
            tokens_parsed: 0,
            token_available: false,
            leading_whitespace: true,
            flow_mapping_started: false,
            implicit_flow_mapping: false,
        }
    }

    /// Get a copy of the last error that was encountered, if any.
    ///
    /// This does not clear the error state and further calls to [`Self::get_error`] will return (a
    /// clone of) the same error.
    #[inline]
    pub fn get_error(&self) -> Option<ScanError> {
        self.error.as_ref().map(std::clone::Clone::clone)
    }

    /// Fill `self.buffer` with at least `count` characters.
    ///
    /// The characters that are extracted this way are not consumed but only placed in the buffer.
    #[inline]
    fn lookahead(&mut self, count: usize) {
        if self.buffer.len() >= count {
            return;
        }
        for _ in 0..(count - self.buffer.len()) {
            self.buffer.push_back(self.rdr.next().unwrap_or('\0'));
        }
    }

    /// Consume the next character, remove it from the buffer and update the mark.
    #[inline]
    fn skip(&mut self) {
        let c = self.buffer.pop_front().unwrap();

        self.mark.index += 1;
        if c == '\n' {
            self.leading_whitespace = true;
            self.mark.line += 1;
            self.mark.col = 0;
        } else {
            // TODO(ethiraric, 20/12/2023): change to `self.leading_whitespace &= is_blank(c)`?
            if self.leading_whitespace && !is_blank(c) {
                self.leading_whitespace = false;
            }
            self.mark.col += 1;
        }
    }

    /// Consume a linebreak (either CR, LF or CRLF), if any. Do nothing if there's none.
    #[inline]
    fn skip_line(&mut self) {
        if self.buffer[0] == '\r' && self.buffer[1] == '\n' {
            self.skip();
            self.skip();
        } else if is_break(self.buffer[0]) {
            self.skip();
        }
    }

    /// Return the next character in the buffer.
    ///
    /// The character is not consumed.
    #[inline]
    fn ch(&self) -> char {
        self.buffer[0]
    }

    /// Look for the next character and return it.
    ///
    /// The character is not consumed.
    /// Equivalent to calling [`Self::lookahead`] and [`Self::ch`].
    #[inline]
    fn look_ch(&mut self) -> char {
        self.lookahead(1);
        self.ch()
    }

    /// Consume and return the next character.
    ///
    /// Equivalent to calling [`Self::ch`] and [`Self::skip`].
    #[inline]
    fn ch_skip(&mut self) -> char {
        let ret = self.ch();
        self.skip();
        ret
    }

    /// Return whether the next character is `c`.
    #[inline]
    fn ch_is(&self, c: char) -> bool {
        self.buffer[0] == c
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

    // Read and consume a line break (either `\r`, `\n` or `\r\n`).
    //
    // A `\n` is pushed into `s`.
    //
    // # Panics
    // If the next characters do not correspond to a line break.
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

    /// Insert a token at the given position.
    fn insert_token(&mut self, pos: usize, tok: Token) {
        let old_len = self.tokens.len();
        assert!(pos <= old_len);
        self.tokens.insert(pos, tok);
    }

    fn allow_simple_key(&mut self) {
        self.simple_key_allowed = true;
    }

    fn disallow_simple_key(&mut self) {
        self.simple_key_allowed = false;
    }

    pub fn fetch_next_token(&mut self) -> ScanResult {
        self.lookahead(1);
        // eprintln!("--> fetch_next_token Cur {:?} {:?}", self.mark, self.ch());

        if !self.stream_start_produced {
            self.fetch_stream_start();
            return Ok(());
        }
        self.skip_to_next_token()?;

        if std::env::var("YAMLRUST_DEBUG").is_ok() {
            eprintln!(
                "  \x1B[38;5;244m\u{2192} fetch_next_token after whitespace {:?} {:?}\x1B[m",
                self.mark,
                self.ch()
            );
        }

        self.stale_simple_keys()?;

        let mark = self.mark;
        self.unroll_indent(mark.col as isize);

        self.lookahead(4);

        if is_z(self.ch()) {
            self.fetch_stream_end()?;
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
            && is_blankz(self.buffer[3])
        {
            self.fetch_document_indicator(TokenType::DocumentStart)?;
            return Ok(());
        }

        if self.mark.col == 0
            && self.buffer[0] == '.'
            && self.buffer[1] == '.'
            && self.buffer[2] == '.'
            && is_blankz(self.buffer[3])
        {
            self.fetch_document_indicator(TokenType::DocumentEnd)?;
            self.skip_ws_to_eol(SkipTabs::Yes);
            if !is_breakz(self.ch()) {
                return Err(ScanError::new(
                    self.mark,
                    "invalid content after document end marker",
                ));
            }
            return Ok(());
        }

        if (self.mark.col as isize) < self.indent {
            return Err(ScanError::new(self.mark, "invalid indentation"));
        }

        let c = self.buffer[0];
        let nc = self.buffer[1];
        match c {
            '[' => self.fetch_flow_collection_start(TokenType::FlowSequenceStart),
            '{' => self.fetch_flow_collection_start(TokenType::FlowMappingStart),
            ']' => self.fetch_flow_collection_end(TokenType::FlowSequenceEnd),
            '}' => self.fetch_flow_collection_end(TokenType::FlowMappingEnd),
            ',' => self.fetch_flow_entry(),
            '-' if is_blankz(nc) => self.fetch_block_entry(),
            '?' if is_blankz(nc) => self.fetch_key(),
            ':' if is_blankz(nc)
                || (self.flow_level > 0
                    && (is_flow(nc) || self.mark.index == self.adjacent_value_allowed_at)) =>
            {
                self.fetch_value()
            }
            // Is it an alias?
            '*' => self.fetch_anchor(true),
            // Is it an anchor?
            '&' => self.fetch_anchor(false),
            '!' => self.fetch_tag(),
            // Is it a literal scalar?
            '|' if self.flow_level == 0 => self.fetch_block_scalar(true),
            // Is it a folded scalar?
            '>' if self.flow_level == 0 => self.fetch_block_scalar(false),
            '\'' => self.fetch_flow_scalar(true),
            '"' => self.fetch_flow_scalar(false),
            // plain scalar
            '-' if !is_blankz(nc) => self.fetch_plain_scalar(),
            ':' | '?' if !is_blankz(nc) && self.flow_level == 0 => self.fetch_plain_scalar(),
            '%' | '@' | '`' => Err(ScanError::new(
                self.mark,
                &format!("unexpected character: `{c}'"),
            )),
            _ => self.fetch_plain_scalar(),
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, ScanError> {
        if self.stream_end_produced {
            return Ok(None);
        }

        if !self.token_available {
            self.fetch_more_tokens()?;
        }
        let t = self.tokens.pop_front().unwrap();
        self.token_available = false;
        self.tokens_parsed += 1;

        if let TokenType::StreamEnd = t.1 {
            self.stream_end_produced = true;
        }
        Ok(Some(t))
    }

    pub fn fetch_more_tokens(&mut self) -> ScanResult {
        let mut need_more;
        loop {
            if self.tokens.is_empty() {
                need_more = true;
            } else {
                need_more = false;
                // Stale potential keys that we know won't be keys.
                self.stale_simple_keys()?;
                // If our next token to be emitted may be a key, fetch more context.
                for sk in &self.simple_keys {
                    if sk.possible && sk.token_number == self.tokens_parsed {
                        need_more = true;
                        break;
                    }
                }
            }

            if !need_more {
                break;
            }
            self.fetch_next_token()?;
        }
        self.token_available = true;

        Ok(())
    }

    /// Mark simple keys that can no longer be keys as such.
    ///
    /// This function sets `possible` to `false` to each key that, now we have more context, we
    /// know will not be keys.
    ///
    /// # Errors
    /// This function returns an error if one of the key we would stale was required to be a key.
    fn stale_simple_keys(&mut self) -> ScanResult {
        for (_, sk) in self.simple_keys.iter_mut().enumerate() {
            if sk.possible
                // If not in a flow construct, simple keys cannot span multiple lines.
                && self.flow_level == 0
                    && (sk.mark.line < self.mark.line || sk.mark.index + 1024 < self.mark.index)
            {
                if sk.required {
                    return Err(ScanError::new(self.mark, "simple key expect ':'"));
                }
                sk.possible = false;
            }
        }
        Ok(())
    }

    /// Skip over all whitespace and comments until the next token.
    ///
    /// # Errors
    /// This function returns an error if a tabulation is encountered where there should not be
    /// one.
    fn skip_to_next_token(&mut self) -> ScanResult {
        loop {
            // TODO(chenyh) BOM
            match self.look_ch() {
                ' ' => self.skip(),
                // Tabs may not be used as indentation.
                // "Indentation" only exists as long as a block is started, but does not exist
                // inside of flow-style constructs. Tabs are allowed as part of leading
                // whitespaces outside of indentation.
                // If a flow-style construct is in an indented block, its contents must still be
                // indented. Also, tabs are allowed anywhere in it if it has no content.
                '\t' if self.is_within_block()
                    && self.leading_whitespace
                    && (self.mark.col as isize) < self.indent =>
                {
                    self.skip_ws_to_eol(SkipTabs::Yes);
                    // If we have content on that line with a tab, return an error.
                    if !is_breakz(self.ch()) {
                        return Err(ScanError::new(
                            self.mark,
                            "tabs disallowed within this context (block indentation)",
                        ));
                    }
                }
                '\t' => self.skip(),
                '\n' | '\r' => {
                    self.lookahead(2);
                    self.skip_line();
                    if self.flow_level == 0 {
                        self.allow_simple_key();
                    }
                }
                '#' => {
                    while !is_breakz(self.ch()) {
                        self.skip();
                        self.lookahead(1);
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Skip over YAML whitespace (` `, `\n`, `\r`).
    ///
    /// # Errors
    /// This function returns an error if no whitespace was found.
    fn skip_yaml_whitespace(&mut self) -> ScanResult {
        let mut need_whitespace = true;
        loop {
            match self.look_ch() {
                ' ' => {
                    self.skip();

                    need_whitespace = false;
                }
                '\n' | '\r' => {
                    self.lookahead(2);
                    self.skip_line();
                    if self.flow_level == 0 {
                        self.allow_simple_key();
                    }
                    need_whitespace = false;
                }
                '#' => {
                    while !is_breakz(self.ch()) {
                        self.skip();
                        self.lookahead(1);
                    }
                }
                _ => break,
            }
        }

        if need_whitespace {
            Err(ScanError::new(self.mark(), "expected whitespace"))
        } else {
            Ok(())
        }
    }

    /// Skip yaml whitespace at most up to eol. Also skips comments.
    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> SkipTabs {
        let mut encountered_tab = false;
        let mut has_yaml_ws = false;
        loop {
            match self.look_ch() {
                ' ' => {
                    has_yaml_ws = true;
                    self.skip();
                }
                '\t' if skip_tabs != SkipTabs::No => {
                    encountered_tab = true;
                    self.skip();
                }
                '#' => {
                    while !is_breakz(self.look_ch()) {
                        self.skip();
                    }
                }
                _ => break,
            }
        }

        SkipTabs::Result(encountered_tab, has_yaml_ws)
    }

    fn fetch_stream_start(&mut self) {
        let mark = self.mark;
        self.indent = -1;
        self.stream_start_produced = true;
        self.allow_simple_key();
        self.tokens
            .push_back(Token(mark, TokenType::StreamStart(TEncoding::Utf8)));
        self.simple_keys.push(SimpleKey::new(Marker::new(0, 0, 0)));
    }

    fn fetch_stream_end(&mut self) -> ScanResult {
        // force new line
        if self.mark.col != 0 {
            self.mark.col = 0;
            self.mark.line += 1;
        }

        // If the stream ended, we won't have more context. We can stall all the simple keys we
        // had. If one was required, however, that was an error and we must propagate it.
        for sk in &mut self.simple_keys {
            if sk.required && sk.possible {
                return Err(ScanError::new(self.mark, "simple key expected"));
            }
            sk.possible = false;
        }

        self.unroll_indent(-1);
        self.remove_simple_key()?;
        self.disallow_simple_key();

        self.tokens
            .push_back(Token(self.mark, TokenType::StreamEnd));
        Ok(())
    }

    fn fetch_directive(&mut self) -> ScanResult {
        self.unroll_indent(-1);
        self.remove_simple_key()?;

        self.disallow_simple_key();

        let tok = self.scan_directive()?;

        self.tokens.push_back(tok);

        Ok(())
    }

    fn scan_directive(&mut self) -> Result<Token, ScanError> {
        let start_mark = self.mark;
        self.skip();

        let name = self.scan_directive_name()?;
        let tok = match name.as_ref() {
            "YAML" => self.scan_version_directive_value(&start_mark)?,
            "TAG" => self.scan_tag_directive_value(&start_mark)?,
            // XXX This should be a warning instead of an error
            _ => {
                // skip current line
                self.lookahead(1);
                while !is_breakz(self.ch()) {
                    self.skip();
                    self.lookahead(1);
                }
                // XXX return an empty TagDirective token
                Token(
                    start_mark,
                    TokenType::TagDirective(String::new(), String::new()),
                )
                // return Err(ScanError::new(start_mark,
                //     "while scanning a directive, found unknown directive name"))
            }
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
            return Err(ScanError::new(
                start_mark,
                "while scanning a directive, did not find expected comment or line break",
            ));
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

        let major = self.scan_version_directive_number(mark)?;

        if self.ch() != '.' {
            return Err(ScanError::new(
                *mark,
                "while scanning a YAML directive, did not find expected digit or '.' character",
            ));
        }

        self.skip();

        let minor = self.scan_version_directive_number(mark)?;

        Ok(Token(*mark, TokenType::VersionDirective(major, minor)))
    }

    fn scan_directive_name(&mut self) -> Result<String, ScanError> {
        let start_mark = self.mark;
        let mut string = String::new();
        self.lookahead(1);
        while is_alpha(self.ch()) {
            string.push(self.ch());
            self.skip();
            self.lookahead(1);
        }

        if string.is_empty() {
            return Err(ScanError::new(
                start_mark,
                "while scanning a directive, could not find expected directive name",
            ));
        }

        if !is_blankz(self.ch()) {
            return Err(ScanError::new(
                start_mark,
                "while scanning a directive, found unexpected non-alphabetical character",
            ));
        }

        Ok(string)
    }

    fn scan_version_directive_number(&mut self, mark: &Marker) -> Result<u32, ScanError> {
        let mut val = 0u32;
        let mut length = 0usize;
        self.lookahead(1);
        while is_digit(self.ch()) {
            if length + 1 > 9 {
                return Err(ScanError::new(
                    *mark,
                    "while scanning a YAML directive, found extremely long version number",
                ));
            }
            length += 1;
            val = val * 10 + ((self.ch() as u32) - ('0' as u32));
            self.skip();
            self.lookahead(1);
        }

        if length == 0 {
            return Err(ScanError::new(
                *mark,
                "while scanning a YAML directive, did not find expected version number",
            ));
        }

        Ok(val)
    }

    fn scan_tag_directive_value(&mut self, mark: &Marker) -> Result<Token, ScanError> {
        self.lookahead(1);
        /* Eat whitespaces. */
        while is_blank(self.ch()) {
            self.skip();
            self.lookahead(1);
        }
        let handle = self.scan_tag_handle(true, mark)?;

        /* Eat whitespaces. */
        while is_blank(self.look_ch()) {
            self.skip();
        }

        let is_secondary = handle == "!!";
        let prefix = self.scan_tag_uri(true, is_secondary, "", mark)?;

        self.lookahead(1);

        if is_blankz(self.ch()) {
            Ok(Token(*mark, TokenType::TagDirective(handle, prefix)))
        } else {
            Err(ScanError::new(
                *mark,
                "while scanning TAG, did not find expected whitespace or line break",
            ))
        }
    }

    fn fetch_tag(&mut self) -> ScanResult {
        self.save_simple_key();
        self.disallow_simple_key();

        let tok = self.scan_tag()?;
        self.tokens.push_back(tok);
        Ok(())
    }

    fn scan_tag(&mut self) -> Result<Token, ScanError> {
        let start_mark = self.mark;
        let mut handle = String::new();
        let mut suffix;

        // Check if the tag is in the canonical form (verbatim).
        self.lookahead(2);

        if self.buffer[1] == '<' {
            // Eat '!<'
            self.skip();
            self.skip();
            suffix = self.scan_tag_uri(false, false, "", &start_mark)?;

            if self.ch() != '>' {
                return Err(ScanError::new(
                    start_mark,
                    "while scanning a tag, did not find the expected '>'",
                ));
            }

            self.skip();
        } else {
            // The tag has either the '!suffix' or the '!handle!suffix'
            handle = self.scan_tag_handle(false, &start_mark)?;
            // Check if it is, indeed, handle.
            if handle.len() >= 2 && handle.starts_with('!') && handle.ends_with('!') {
                // A tag handle starting with "!!" is a secondary tag handle.
                let is_secondary_handle = handle == "!!";
                suffix = self.scan_tag_uri(false, is_secondary_handle, "", &start_mark)?;
            } else {
                suffix = self.scan_tag_uri(false, false, &handle, &start_mark)?;
                handle = "!".to_owned();
                // A special case: the '!' tag.  Set the handle to '' and the
                // suffix to '!'.
                if suffix.is_empty() {
                    handle.clear();
                    suffix = "!".to_owned();
                }
            }
        }

        if is_blankz(self.look_ch()) {
            // XXX: ex 7.2, an empty scalar can follow a secondary tag
            Ok(Token(start_mark, TokenType::Tag(handle, suffix)))
        } else {
            Err(ScanError::new(
                start_mark,
                "while scanning a tag, did not find expected whitespace or line break",
            ))
        }
    }

    fn scan_tag_handle(&mut self, directive: bool, mark: &Marker) -> Result<String, ScanError> {
        let mut string = String::new();
        if self.look_ch() != '!' {
            return Err(ScanError::new(
                *mark,
                "while scanning a tag, did not find expected '!'",
            ));
        }

        string.push(self.ch_skip());

        while is_alpha(self.look_ch()) {
            string.push(self.ch_skip());
        }

        // Check if the trailing character is '!' and copy it.
        if self.ch() == '!' {
            string.push(self.ch_skip());
        } else if directive && string != "!" {
            // It's either the '!' tag or not really a tag handle.  If it's a %TAG
            // directive, it's an error.  If it's a tag token, it must be a part of
            // URI.
            return Err(ScanError::new(
                *mark,
                "while parsing a tag directive, did not find expected '!'",
            ));
        }
        Ok(string)
    }

    fn scan_tag_uri(
        &mut self,
        directive: bool,
        _is_secondary: bool,
        head: &str,
        mark: &Marker,
    ) -> Result<String, ScanError> {
        let mut length = head.len();
        let mut string = String::new();

        // Copy the head if needed.
        // Note that we don't copy the leading '!' character.
        if length > 1 {
            string.extend(head.chars().skip(1));
        }

        /*
         * The set of characters that may appear in URI is as follows:
         *
         *      '0'-'9', 'A'-'Z', 'a'-'z', '_', '-', ';', '/', '?', ':', '@', '&',
         *      '=', '+', '$', ',', '.', '!', '~', '*', '\'', '(', ')', '[', ']',
         *      '%'.
         */
        while match self.look_ch() {
            ';' | '/' | '?' | ':' | '@' | '&' => true,
            '=' | '+' | '$' | ',' | '.' | '!' | '~' | '*' | '\'' | '(' | ')' | '[' | ']' => true,
            '%' => true,
            c if is_alpha(c) => true,
            _ => false,
        } {
            // Check if it is a URI-escape sequence.
            if self.ch() == '%' {
                string.push(self.scan_uri_escapes(directive, mark)?);
            } else {
                string.push(self.ch());
                self.skip();
            }

            length += 1;
        }

        if length == 0 {
            return Err(ScanError::new(
                *mark,
                "while parsing a tag, did not find expected tag URI",
            ));
        }

        Ok(string)
    }

    fn scan_uri_escapes(&mut self, _directive: bool, mark: &Marker) -> Result<char, ScanError> {
        let mut width = 0usize;
        let mut code = 0u32;
        loop {
            self.lookahead(3);

            if !(self.ch() == '%' && is_hex(self.buffer[1]) && is_hex(self.buffer[2])) {
                return Err(ScanError::new(
                    *mark,
                    "while parsing a tag, did not find URI escaped octet",
                ));
            }

            let octet = (as_hex(self.buffer[1]) << 4) + as_hex(self.buffer[2]);
            if width == 0 {
                width = match octet {
                    _ if octet & 0x80 == 0x00 => 1,
                    _ if octet & 0xE0 == 0xC0 => 2,
                    _ if octet & 0xF0 == 0xE0 => 3,
                    _ if octet & 0xF8 == 0xF0 => 4,
                    _ => {
                        return Err(ScanError::new(
                            *mark,
                            "while parsing a tag, found an incorrect leading UTF-8 octet",
                        ));
                    }
                };
                code = octet;
            } else {
                if octet & 0xc0 != 0x80 {
                    return Err(ScanError::new(
                        *mark,
                        "while parsing a tag, found an incorrect trailing UTF-8 octet",
                    ));
                }
                code = (code << 8) + octet;
            }

            self.skip();
            self.skip();
            self.skip();

            width -= 1;
            if width == 0 {
                break;
            }
        }

        match char::from_u32(code) {
            Some(ch) => Ok(ch),
            None => Err(ScanError::new(
                *mark,
                "while parsing a tag, found an invalid UTF-8 codepoint",
            )),
        }
    }

    fn fetch_anchor(&mut self, alias: bool) -> ScanResult {
        self.save_simple_key();
        self.disallow_simple_key();

        let tok = self.scan_anchor(alias)?;

        self.tokens.push_back(tok);

        Ok(())
    }

    fn scan_anchor(&mut self, alias: bool) -> Result<Token, ScanError> {
        let mut string = String::new();
        let start_mark = self.mark;

        self.skip();
        while is_anchor_char(self.look_ch()) {
            string.push(self.ch());
            self.skip();
        }

        if string.is_empty() {
            return Err(ScanError::new(start_mark, "while scanning an anchor or alias, did not find expected alphabetic or numeric character"));
        }

        if alias {
            Ok(Token(start_mark, TokenType::Alias(string)))
        } else {
            Ok(Token(start_mark, TokenType::Anchor(string)))
        }
    }

    fn fetch_flow_collection_start(&mut self, tok: TokenType) -> ScanResult {
        // The indicators '[' and '{' may start a simple key.
        self.save_simple_key();

        self.roll_one_col_indent();
        self.increase_flow_level()?;

        self.allow_simple_key();

        let start_mark = self.mark;
        self.skip();

        if tok == TokenType::FlowMappingStart {
            self.flow_mapping_started = true;
        }

        self.tokens.push_back(Token(start_mark, tok));
        Ok(())
    }

    fn fetch_flow_collection_end(&mut self, tok: TokenType) -> ScanResult {
        self.remove_simple_key()?;
        self.decrease_flow_level();

        self.disallow_simple_key();

        self.end_implicit_mapping(self.mark);

        let start_mark = self.mark;
        self.skip();

        self.tokens.push_back(Token(start_mark, tok));
        Ok(())
    }

    /// Push the `FlowEntry` token and skip over the `,`.
    fn fetch_flow_entry(&mut self) -> ScanResult {
        self.remove_simple_key()?;
        self.allow_simple_key();

        self.end_implicit_mapping(self.mark);

        let start_mark = self.mark;
        self.skip();

        self.tokens
            .push_back(Token(start_mark, TokenType::FlowEntry));
        Ok(())
    }

    fn increase_flow_level(&mut self) -> ScanResult {
        self.simple_keys.push(SimpleKey::new(Marker::new(0, 0, 0)));
        self.flow_level = self
            .flow_level
            .checked_add(1)
            .ok_or_else(|| ScanError::new(self.mark, "recursion limit exceeded"))?;
        Ok(())
    }

    fn decrease_flow_level(&mut self) {
        if self.flow_level > 0 {
            self.flow_level -= 1;
            self.simple_keys.pop().unwrap();
        }
    }

    /// Push the `Block*` token(s) and skip over the `-`.
    ///
    /// Add an indentation level and push a `BlockSequenceStart` token if needed, then push a
    /// `BlockEntry` token.
    /// This function only skips over the `-` and does not fetch the entry value.
    fn fetch_block_entry(&mut self) -> ScanResult {
        if self.flow_level > 0 {
            // - * only allowed in block
            return Err(ScanError::new(
                self.mark,
                r#""-" is only valid inside a block"#,
            ));
        }
        // Check if we are allowed to start a new entry.
        if !self.simple_key_allowed {
            return Err(ScanError::new(
                self.mark,
                "block sequence entries are not allowed in this context",
            ));
        }

        // Skip over the `-`.
        let mark = self.mark;
        self.skip();

        // generate BLOCK-SEQUENCE-START if indented
        self.roll_indent(mark.col, None, TokenType::BlockSequenceStart, mark);
        let found_tabs = self.skip_ws_to_eol(SkipTabs::Yes).found_tabs();
        self.lookahead(2);
        if found_tabs && self.buffer[0] == '-' && is_blankz(self.buffer[1]) {
            return Err(ScanError::new(
                self.mark,
                "'-' must be followed by a valid YAML whitespace",
            ));
        }

        self.skip_ws_to_eol(SkipTabs::No);
        if is_break(self.look_ch()) || is_flow(self.ch()) {
            self.roll_one_col_indent();
        }

        self.remove_simple_key()?;
        self.allow_simple_key();

        self.tokens
            .push_back(Token(self.mark, TokenType::BlockEntry));

        Ok(())
    }

    fn fetch_document_indicator(&mut self, t: TokenType) -> ScanResult {
        self.unroll_indent(-1);
        self.remove_simple_key()?;
        self.disallow_simple_key();

        let mark = self.mark;

        self.skip();
        self.skip();
        self.skip();

        self.tokens.push_back(Token(mark, t));
        Ok(())
    }

    fn fetch_block_scalar(&mut self, literal: bool) -> ScanResult {
        self.save_simple_key();
        self.allow_simple_key();
        let tok = self.scan_block_scalar(literal)?;

        self.tokens.push_back(tok);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
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
        self.unroll_non_block_indents();

        if self.look_ch() == '+' || self.ch() == '-' {
            if self.ch() == '+' {
                chomping = 1;
            } else {
                chomping = -1;
            }
            self.skip();
            if is_digit(self.look_ch()) {
                if self.ch() == '0' {
                    return Err(ScanError::new(
                        start_mark,
                        "while scanning a block scalar, found an indentation indicator equal to 0",
                    ));
                }
                increment = (self.ch() as usize) - ('0' as usize);
                self.skip();
            }
        } else if is_digit(self.ch()) {
            if self.ch() == '0' {
                return Err(ScanError::new(
                    start_mark,
                    "while scanning a block scalar, found an indentation indicator equal to 0",
                ));
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

        self.skip_ws_to_eol(SkipTabs::Yes);

        // Check if we are at the end of the line.
        if !is_breakz(self.ch()) {
            return Err(ScanError::new(
                start_mark,
                "while scanning a block scalar, did not find expected comment or line break",
            ));
        }

        if is_break(self.ch()) {
            self.lookahead(2);
            self.skip_line();
        }

        if self.look_ch() == '\t' {
            return Err(ScanError::new(
                start_mark,
                "a block scalar content cannot start with a tab",
            ));
        }

        if increment > 0 {
            indent = if self.indent >= 0 {
                (self.indent + increment as isize) as usize
            } else {
                increment
            }
        }

        // Scan the leading line breaks and determine the indentation level if needed.
        if indent == 0 {
            self.skip_block_scalar_first_line_indent(&mut indent, &mut trailing_breaks);
        } else {
            self.skip_block_scalar_indent(indent, &mut trailing_breaks);
        }

        self.lookahead(1);

        let start_mark = self.mark;

        while self.mark.col == indent && !is_z(self.ch()) {
            // We are at the beginning of a non-empty line.
            trailing_blank = is_blank(self.ch());
            if !literal && !leading_break.is_empty() && !leading_blank && !trailing_blank {
                if trailing_breaks.is_empty() {
                    string.push(' ');
                }
                leading_break.clear();
            } else {
                string.push_str(&leading_break);
                leading_break.clear();
            }

            string.push_str(&trailing_breaks);
            trailing_breaks.clear();

            leading_blank = is_blank(self.ch());

            while !is_breakz(self.ch()) {
                string.push(self.ch());
                self.skip();
                self.lookahead(1);
            }
            // break on EOF
            if is_z(self.ch()) {
                break;
            }

            self.lookahead(2);
            self.read_break(&mut leading_break);

            // Eat the following indentation spaces and line breaks.
            self.skip_block_scalar_indent(indent, &mut trailing_breaks);
        }

        // Chomp the tail.
        if chomping != -1 {
            string.push_str(&leading_break);
        }

        if chomping == 1 {
            string.push_str(&trailing_breaks);
        }

        if literal {
            Ok(Token(
                start_mark,
                TokenType::Scalar(TScalarStyle::Literal, string),
            ))
        } else {
            Ok(Token(
                start_mark,
                TokenType::Scalar(TScalarStyle::Foled, string),
            ))
        }
    }

    /// Skip the block scalar indentation and empty lines.
    fn skip_block_scalar_indent(&mut self, indent: usize, breaks: &mut String) {
        loop {
            // Consume all spaces. Tabs cannot be used as indentation.
            while self.mark.col < indent && self.look_ch() == ' ' {
                self.skip();
            }

            // If our current line is empty, skip over the break and continue looping.
            if is_break(self.look_ch()) {
                self.lookahead(2);
                self.read_break(breaks);
            } else {
                // Otherwise, we have a content line. Return control.
                break;
            }
        }
    }

    /// Determine the indentation level for a block scalar from the first line of its contents.
    ///
    /// The function skips over whitespace-only lines and sets `indent` to the the longest
    /// whitespace line that was encountered.
    fn skip_block_scalar_first_line_indent(&mut self, indent: &mut usize, breaks: &mut String) {
        let mut max_indent = 0;
        loop {
            // Consume all spaces. Tabs cannot be used as indentation.
            while self.look_ch() == ' ' {
                self.skip();
            }

            if self.mark.col > max_indent {
                max_indent = self.mark.col;
            }

            if is_break(self.look_ch()) {
                // If our current line is empty, skip over the break and continue looping.
                self.lookahead(2);
                self.read_break(breaks);
            } else {
                // Otherwise, we have a content line. Return control.
                break;
            }
        }

        // In case a yaml looks like:
        // ```yaml
        // |
        // foo
        // bar
        // ```
        // We need to set the indent to 0 and not 1. In all other cases, the indent must be at
        // least 1. When in the above example, `self.indent` will be set to -1.
        *indent = max_indent.max((self.indent + 1) as usize);
        if self.indent > 0 {
            *indent = (*indent).max(1);
        }
    }

    fn fetch_flow_scalar(&mut self, single: bool) -> ScanResult {
        self.save_simple_key();
        self.disallow_simple_key();

        let tok = self.scan_flow_scalar(single)?;

        // From spec: To ensure JSON compatibility, if a key inside a flow mapping is JSON-like,
        // YAML allows the following value to be specified adjacent to the “:”.
        self.skip_to_next_token()?;
        self.adjacent_value_allowed_at = self.mark.index;

        self.tokens.push_back(tok);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
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

            if self.mark.col == 0
                && (((self.buffer[0] == '-') && (self.buffer[1] == '-') && (self.buffer[2] == '-'))
                    || ((self.buffer[0] == '.')
                        && (self.buffer[1] == '.')
                        && (self.buffer[2] == '.')))
                && is_blankz(self.buffer[3])
            {
                return Err(ScanError::new(
                    start_mark,
                    "while scanning a quoted scalar, found unexpected document indicator",
                ));
            }

            if is_z(self.ch()) {
                return Err(ScanError::new(
                    start_mark,
                    "while scanning a quoted scalar, found unexpected end of stream",
                ));
            }

            leading_blanks = false;
            self.consume_flow_scalar_non_whitespace_chars(
                single,
                &mut string,
                &mut leading_blanks,
                &start_mark,
            )?;

            match self.look_ch() {
                '\'' if single => break,
                '"' if !single => break,
                _ => {}
            }

            // Consume blank characters.
            while is_blank(self.ch()) || is_break(self.ch()) {
                if is_blank(self.ch()) {
                    // Consume a space or a tab character.
                    if leading_blanks {
                        if self.ch() == '\t' && (self.mark.col as isize) < self.indent {
                            return Err(ScanError::new(
                                self.mark,
                                "tab cannot be used as indentation",
                            ));
                        }
                        self.skip();
                    } else {
                        whitespaces.push(self.ch());
                        self.skip();
                    }
                } else {
                    self.lookahead(2);
                    // Check if it is a first line break.
                    if leading_blanks {
                        self.read_break(&mut trailing_breaks);
                    } else {
                        whitespaces.clear();
                        self.read_break(&mut leading_break);
                        leading_blanks = true;
                    }
                }
                self.lookahead(1);
            }

            // Join the whitespaces or fold line breaks.
            if leading_blanks {
                if leading_break.is_empty() {
                    string.push_str(&leading_break);
                    string.push_str(&trailing_breaks);
                    trailing_breaks.clear();
                    leading_break.clear();
                } else {
                    if trailing_breaks.is_empty() {
                        string.push(' ');
                    } else {
                        string.push_str(&trailing_breaks);
                        trailing_breaks.clear();
                    }
                    leading_break.clear();
                }
            } else {
                string.push_str(&whitespaces);
                whitespaces.clear();
            }
        } // loop

        // Eat the right quote.
        self.skip();
        // Ensure there is no invalid trailing content.
        self.skip_ws_to_eol(SkipTabs::Yes);
        match self.ch() {
            // These can be encountered in flow sequences or mappings.
            ',' | '}' | ']' if self.flow_level > 0 => {}
            // An end-of-line / end-of-stream is fine. No trailing content.
            c if is_breakz(c) => {}
            // ':' can be encountered if our scalar is a key.
            // Outside of flow contexts, keys cannot span multiple lines
            ':' if self.flow_level == 0 && start_mark.line == self.mark.line => {}
            // Inside a flow context, this is allowed.
            ':' if self.flow_level > 0 => {}
            _ => {
                return Err(ScanError::new(
                    self.mark,
                    "invalid trailing content after double-quoted scalar",
                ));
            }
        }

        let style = if single {
            TScalarStyle::SingleQuoted
        } else {
            TScalarStyle::DoubleQuoted
        };
        Ok(Token(start_mark, TokenType::Scalar(style, string)))
    }

    /// Consume successive non-whitespace characters from a flow scalar.
    ///
    /// This function resolves escape sequences and stops upon encountering a whitespace, the end
    /// of the stream or the closing character for the scalar (`'` for single quoted scalars, `"`
    /// for double quoted scalars).
    ///
    /// # Errors
    /// Return an error if an invalid escape sequence is found.
    fn consume_flow_scalar_non_whitespace_chars(
        &mut self,
        single: bool,
        string: &mut String,
        leading_blanks: &mut bool,
        start_mark: &Marker,
    ) -> Result<(), ScanError> {
        self.lookahead(2);
        while !is_blankz(self.ch()) {
            match self.ch() {
                // Check for an escaped single quote.
                '\'' if self.buffer[1] == '\'' && single => {
                    string.push('\'');
                    self.skip();
                    self.skip();
                }
                // Check for the right quote.
                '\'' if single => break,
                '"' if !single => break,
                // Check for an escaped line break.
                '\\' if !single && is_break(self.buffer[1]) => {
                    self.lookahead(3);
                    self.skip();
                    self.skip_line();
                    *leading_blanks = true;
                    break;
                }
                // Check for an escape sequence.
                '\\' if !single => {
                    string.push(self.resolve_flow_scalar_escape_sequence(start_mark)?);
                }
                c => {
                    string.push(c);
                    self.skip();
                }
            }
            self.lookahead(2);
        }
        Ok(())
    }

    /// Escape the sequence we encounter in a flow scalar.
    ///
    /// `self.ch()` must point to the `\` starting the escape sequence.
    ///
    /// # Errors
    /// Return an error if an invalid escape sequence is found.
    fn resolve_flow_scalar_escape_sequence(
        &mut self,
        start_mark: &Marker,
    ) -> Result<char, ScanError> {
        let mut code_length = 0usize;
        let mut ret = '\0';

        match self.buffer[1] {
            '0' => ret = '\0',
            'a' => ret = '\x07',
            'b' => ret = '\x08',
            't' | '\t' => ret = '\t',
            'n' => ret = '\n',
            'v' => ret = '\x0b',
            'f' => ret = '\x0c',
            'r' => ret = '\x0d',
            'e' => ret = '\x1b',
            ' ' => ret = '\x20',
            '"' => ret = '"',
            '\'' => ret = '\'',
            '\\' => ret = '\\',
            // Unicode next line (#x85)
            'N' => ret = char::from_u32(0x85).unwrap(),
            // Unicode non-breaking space (#xA0)
            '_' => ret = char::from_u32(0xA0).unwrap(),
            // Unicode line separator (#x2028)
            'L' => ret = char::from_u32(0x2028).unwrap(),
            // Unicode paragraph separator (#x2029)
            'P' => ret = char::from_u32(0x2029).unwrap(),
            'x' => code_length = 2,
            'u' => code_length = 4,
            'U' => code_length = 8,
            _ => {
                return Err(ScanError::new(
                    *start_mark,
                    "while parsing a quoted scalar, found unknown escape character",
                ))
            }
        }
        self.skip();
        self.skip();

        // Consume an arbitrary escape code.
        if code_length > 0 {
            self.lookahead(code_length);
            let mut value = 0u32;
            for i in 0..code_length {
                if !is_hex(self.buffer[i]) {
                    return Err(ScanError::new(
                        *start_mark,
                        "while parsing a quoted scalar, did not find expected hexadecimal number",
                    ));
                }
                value = (value << 4) + as_hex(self.buffer[i]);
            }

            let Some(ch) = char::from_u32(value) else {
                return Err(ScanError::new(
                    *start_mark,
                    "while parsing a quoted scalar, found invalid Unicode character escape code",
                ));
            };
            ret = ch;

            for _ in 0..code_length {
                self.skip();
            }
        }
        Ok(ret)
    }

    fn fetch_plain_scalar(&mut self) -> ScanResult {
        self.save_simple_key();
        self.disallow_simple_key();

        let tok = self.scan_plain_scalar()?;

        self.tokens.push_back(tok);
        Ok(())
    }

    fn scan_plain_scalar(&mut self) -> Result<Token, ScanError> {
        self.unroll_non_block_indents();
        let indent = self.indent + 1;
        let start_mark = self.mark;

        let mut string = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        let mut whitespaces = String::new();
        let mut leading_blanks = true;

        loop {
            /* Check for a document indicator. */
            self.lookahead(4);

            if self.mark.col == 0
                && (((self.buffer[0] == '-') && (self.buffer[1] == '-') && (self.buffer[2] == '-'))
                    || ((self.buffer[0] == '.')
                        && (self.buffer[1] == '.')
                        && (self.buffer[2] == '.')))
                && is_blankz(self.buffer[3])
            {
                break;
            }

            if self.ch() == '#' {
                break;
            }
            while !is_blankz(self.ch()) {
                // indicators can end a plain scalar, see 7.3.3. Plain Style
                match self.ch() {
                    ':' if is_blankz(self.buffer[1])
                        || (self.flow_level > 0 && is_flow(self.buffer[1])) =>
                    {
                        break;
                    }
                    ',' | '[' | ']' | '{' | '}' if self.flow_level > 0 => break,
                    _ => {}
                }

                if leading_blanks || !whitespaces.is_empty() {
                    if leading_blanks {
                        if leading_break.is_empty() {
                            string.push_str(&leading_break);
                            string.push_str(&trailing_breaks);
                            trailing_breaks.clear();
                            leading_break.clear();
                        } else {
                            if trailing_breaks.is_empty() {
                                string.push(' ');
                            } else {
                                string.push_str(&trailing_breaks);
                                trailing_breaks.clear();
                            }
                            leading_break.clear();
                        }
                        leading_blanks = false;
                    } else {
                        string.push_str(&whitespaces);
                        whitespaces.clear();
                    }
                }

                string.push(self.ch());
                self.skip();
                self.lookahead(2);
            }
            // is the end?
            if !(is_blank(self.ch()) || is_break(self.ch())) {
                break;
            }

            while is_blank(self.look_ch()) || is_break(self.ch()) {
                if is_blank(self.ch()) {
                    if leading_blanks && (self.mark.col as isize) < indent && self.ch() == '\t' {
                        // If our line contains only whitespace, this is not an error.
                        // Skip over it.
                        self.skip_ws_to_eol(SkipTabs::Yes);
                        if is_breakz(self.ch()) {
                            continue;
                        }
                        return Err(ScanError::new(
                            start_mark,
                            "while scanning a plain scalar, found a tab",
                        ));
                    }

                    if !leading_blanks {
                        whitespaces.push(self.ch());
                    }
                    self.skip();
                } else {
                    self.lookahead(2);
                    // Check if it is a first line break
                    if leading_blanks {
                        self.read_break(&mut trailing_breaks);
                    } else {
                        whitespaces.clear();
                        self.read_break(&mut leading_break);
                        leading_blanks = true;
                    }
                }
            }

            // check indentation level
            if self.flow_level == 0 && (self.mark.col as isize) < indent {
                break;
            }
        }

        if leading_blanks {
            self.allow_simple_key();
        }

        Ok(Token(
            start_mark,
            TokenType::Scalar(TScalarStyle::Plain, string),
        ))
    }

    fn fetch_key(&mut self) -> ScanResult {
        let start_mark = self.mark;
        if self.flow_level == 0 {
            // Check if we are allowed to start a new key (not necessarily simple).
            if !self.simple_key_allowed {
                return Err(ScanError::new(
                    self.mark,
                    "mapping keys are not allowed in this context",
                ));
            }
            self.roll_indent(
                start_mark.col,
                None,
                TokenType::BlockMappingStart,
                start_mark,
            );
        } else {
            // The parser, upon receiving a `Key`, will insert a `MappingStart` event.
            self.flow_mapping_started = true;
        }

        self.remove_simple_key()?;

        if self.flow_level == 0 {
            self.allow_simple_key();
        } else {
            self.disallow_simple_key();
        }

        self.skip();
        self.skip_yaml_whitespace()?;
        if self.ch() == '\t' {
            return Err(ScanError::new(
                self.mark(),
                "tabs disallowed in this context",
            ));
        }
        self.tokens.push_back(Token(start_mark, TokenType::Key));
        Ok(())
    }

    /// Fetch a value from a mapping (after a `:`).
    fn fetch_value(&mut self) -> ScanResult {
        let sk = self.simple_keys.last().unwrap().clone();
        let start_mark = self.mark;
        self.implicit_flow_mapping = self.flow_level > 0 && !self.flow_mapping_started;

        // Skip over ':'.
        self.skip();
        if self.look_ch() == '\t'
            && !self.skip_ws_to_eol(SkipTabs::Yes).has_valid_yaml_ws()
            && (self.ch() == '-' || is_alpha(self.ch()))
        {
            return Err(ScanError::new(
                self.mark,
                "':' must be followed by a valid YAML whitespace",
            ));
        }

        if sk.possible {
            // insert simple key
            let tok = Token(sk.mark, TokenType::Key);
            self.insert_token(sk.token_number - self.tokens_parsed, tok);
            if self.implicit_flow_mapping {
                if sk.mark.line < start_mark.line {
                    return Err(ScanError::new(
                        start_mark,
                        "illegal placement of ':' indicator",
                    ));
                }
                self.insert_token(
                    sk.token_number - self.tokens_parsed,
                    Token(self.mark, TokenType::FlowMappingStart),
                );
            }

            // Add the BLOCK-MAPPING-START token if needed.
            self.roll_indent(
                sk.mark.col,
                Some(sk.token_number),
                TokenType::BlockMappingStart,
                start_mark,
            );
            self.roll_one_col_indent();

            self.simple_keys.last_mut().unwrap().possible = false;
            self.disallow_simple_key();
        } else {
            if self.implicit_flow_mapping {
                self.tokens
                    .push_back(Token(self.mark, TokenType::FlowMappingStart));
            }
            // The ':' indicator follows a complex key.
            if self.flow_level == 0 {
                if !self.simple_key_allowed {
                    return Err(ScanError::new(
                        start_mark,
                        "mapping values are not allowed in this context",
                    ));
                }

                self.roll_indent(
                    start_mark.col,
                    None,
                    TokenType::BlockMappingStart,
                    start_mark,
                );
            }
            self.roll_one_col_indent();

            if self.flow_level == 0 {
                self.allow_simple_key();
            } else {
                self.disallow_simple_key();
            }
        }
        self.tokens.push_back(Token(start_mark, TokenType::Value));

        Ok(())
    }

    /// Add an indentation level to the stack with the given block token, if needed.
    ///
    /// An indentation level is added only if:
    ///   - We are not in a flow-style construct (which don't have indentation per-se).
    ///   - The current column is further indented than the last indent we have registered.
    fn roll_indent(&mut self, col: usize, number: Option<usize>, tok: TokenType, mark: Marker) {
        if self.flow_level > 0 {
            return;
        }

        // If the last indent was a non-block indent, remove it.
        // This means that we prepared an indent that we thought we wouldn't use, but realized just
        // now that it is a block indent.
        if self.indent <= col as isize {
            if let Some(indent) = self.indents.last() {
                if !indent.needs_block_end {
                    self.indent = indent.indent;
                    self.indents.pop();
                }
            }
        }

        if self.indent < col as isize {
            self.indents.push(Indent {
                indent: self.indent,
                needs_block_end: true,
            });
            self.indent = col as isize;
            let tokens_parsed = self.tokens_parsed;
            match number {
                Some(n) => self.insert_token(n - tokens_parsed, Token(mark, tok)),
                None => self.tokens.push_back(Token(mark, tok)),
            }
        }
    }

    /// Pop indentation levels from the stack as much as needed.
    ///
    /// Indentation levels are popped from the stack while they are further indented than `col`.
    /// If we are in a flow-style construct (which don't have indentation per-se), this function
    /// does nothing.
    fn unroll_indent(&mut self, col: isize) {
        if self.flow_level > 0 {
            return;
        }
        while self.indent > col {
            let indent = self.indents.pop().unwrap();
            self.indent = indent.indent;
            if indent.needs_block_end {
                self.tokens.push_back(Token(self.mark, TokenType::BlockEnd));
            }
        }
    }

    /// Add an indentation level of 1 column that does not start a block.
    ///
    /// See the documentation of [`Indent::needs_block_end`] for more details.
    /// An indentation is not added if we are inside a flow level or if the last indent is already
    /// a non-block indent.
    fn roll_one_col_indent(&mut self) {
        if self.flow_level == 0 && self.indents.last().map_or(false, |x| x.needs_block_end) {
            self.indents.push(Indent {
                indent: self.indent,
                needs_block_end: false,
            });
            self.indent += 1;
        }
    }

    /// Unroll all last indents created with [`Self::roll_one_col_indent`].
    fn unroll_non_block_indents(&mut self) {
        while let Some(indent) = self.indents.last() {
            if indent.needs_block_end {
                break;
            } else {
                self.indent = indent.indent;
                self.indents.pop();
            }
        }
    }

    /// Save the last token in [`Self::tokens`] as a simple key.
    fn save_simple_key(&mut self) {
        if self.simple_key_allowed {
            let required = self.flow_level > 0
                && self.indent == (self.mark.col as isize)
                && self.indents.last().unwrap().needs_block_end;
            let mut sk = SimpleKey::new(self.mark);
            sk.possible = true;
            sk.required = required;
            sk.token_number = self.tokens_parsed + self.tokens.len();

            self.simple_keys.pop();
            self.simple_keys.push(sk);
        }
    }

    fn remove_simple_key(&mut self) -> ScanResult {
        let last = self.simple_keys.last_mut().unwrap();
        if last.possible && last.required {
            return Err(ScanError::new(self.mark, "simple key expected"));
        }

        last.possible = false;
        Ok(())
    }

    /// Return whether the scanner is inside a block but outside of a flow sequence.
    fn is_within_block(&self) -> bool {
        !self.indents.is_empty()
    }

    /// If an implicit mapping had started, end it.
    fn end_implicit_mapping(&mut self, mark: Marker) {
        if self.implicit_flow_mapping {
            self.implicit_flow_mapping = false;
            self.flow_mapping_started = false;
            self.tokens
                .push_back(Token(mark, TokenType::FlowMappingEnd));
        }
    }
}

/// Behavior to adopt regarding treating tabs as whitespace.
///
/// Although tab is a valid yaml whitespace, it doesn't always behave the same as a space.
#[derive(Copy, Clone, Eq, PartialEq)]
enum SkipTabs {
    /// Skip all tabs as whitespace.
    Yes,
    /// Don't skip any tab. Return from the function when encountering one.
    No,
    /// Return value from the function.
    Result(
        /// Whether tabs were encountered.
        bool,
        /// Whether at least 1 valid yaml whitespace has been encountered.
        bool,
    ),
}

impl SkipTabs {
    /// Whether tabs were found while skipping whitespace.
    ///
    /// This function must be called after a call to `skip_ws_to_eol`.
    fn found_tabs(self) -> bool {
        matches!(self, SkipTabs::Result(true, _))
    }

    /// Whether a valid YAML whitespace has been found in skipped-over content.
    ///
    /// This function must be called after a call to `skip_ws_to_eol`.
    fn has_valid_yaml_ws(self) -> bool {
        matches!(self, SkipTabs::Result(_, true))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_is_anchor_char() {
        use super::is_anchor_char;
        assert!(is_anchor_char('x'));
    }
}
