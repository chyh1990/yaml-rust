use scanner::*;
use yaml::*;

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum State {
    StreamStart,
    ImplicitDocumentStart,
    DocumentStart,
    DocumentContent,
    DocumentEnd,
    BlockNode,
    BlockNodeOrIndentlessSequence,
    FlowNode,
    BlockSequenceFirstEntry,
    BlockSequenceEntry,
    IndentlessSequenceEntry,
    BlockMappingFirstKey,
    BlockMappingKey,
    BlockMappingValue,
    FlowSequenceFirstEntry,
    FlowSequenceEntry,
    FlowSequenceEntryMappingKey,
    FlowSequenceEntryMappingValue,
    FlowSequenceEntryMappingEnd,
    FlowMappingFirstKey,
    FlowMappingKey,
    FlowMappingValue,
    FlowMappingEmptyValue,
    End
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Event {
    NoEvent,
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Alias,
    Scalar(String),
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd
}

impl Event {
    fn empty_scalar() -> Event {
        Event::Scalar(String::new())
    }
}

#[derive(Debug)]
pub struct Parser<T> {
    scanner: Scanner<T>,
    states: Vec<State>,
    state: State,
    marks: Vec<Marker>,
    token: Option<Token>,
}

pub type ParseResult = Result<Event, ScanError>;

impl<T: Iterator<Item=char>> Parser<T> {
    pub fn new(src: T) -> Parser<T> {
        Parser {
            scanner: Scanner::new(src),
            states: Vec::new(),
            state: State::StreamStart,
            marks: Vec::new(),
            token: None,
        }
    }

    fn peek(&mut self) -> Result<Token, ScanError> {
        if self.token.is_none() {
            self.token = self.scanner.next();
        }
        if self.token.is_none() {
            return Err(ScanError::new(self.scanner.mark(), 
                                      "unexpected eof"));
        }
        // XXX better?
        Ok(self.token.clone().unwrap())
    }

    fn skip(&mut self) {
        self.token = None;
        //self.peek();
    }
    fn pop_state(&mut self) {
        self.state = self.states.pop().unwrap()
    }
    fn push_state(&mut self, state: State) {
        self.states.push(state);
    }

    pub fn parse(&mut self) -> ParseResult {
        if self.scanner.stream_ended()
            || self.state == State::End {
            return Ok(Event::NoEvent);
        }
        let ev = self.state_machine();
        println!("EV {:?}", ev);
        ev
    }

    pub fn load(&mut self) -> Result<Yaml, ScanError> {
        if !self.scanner.stream_started() {
            let ev = try!(self.parse());
            assert_eq!(ev, Event::StreamStart);
        }

        if self.scanner.stream_ended() {
            return Ok(Yaml::Null);
        }
        let ev = try!(self.parse());
        if ev == Event::StreamEnd {
            return Ok(Yaml::Null);
        }
        self.load_document(&ev)
    }

    fn load_document(&mut self, first_ev: &Event) -> Result<Yaml, ScanError> {
        assert_eq!(first_ev, &Event::DocumentStart);

        let ev = try!(self.parse());
        self.load_node(&ev)
    }

    fn load_node(&mut self, first_ev: &Event) -> Result<Yaml, ScanError> {
        match *first_ev {
            Event::Scalar(ref v) => {
                // TODO scalar
                println!("Scalar: {:?}", first_ev);
                Ok(Yaml::String(v.clone()))
            },
            Event::SequenceStart => {
                self.load_sequence(first_ev)
            },
            Event::MappingStart => {
                self.load_mapping(first_ev)
            },
            _ => { unreachable!(); }
        }
    }

    fn load_mapping(&mut self, first_ev: &Event) -> Result<Yaml, ScanError> {
        let mut ev = try!(self.parse());
        let mut map = Hash::new();
        while ev != Event::MappingEnd {
            // key
            let key = try!(self.load_node(&ev));

            // value
            ev = try!(self.parse());
            let value = try!(self.load_node(&ev));

            map.insert(key, value);

            // next event
            ev = try!(self.parse());
        }
        Ok(Yaml::Hash(map))
    }

    fn load_sequence(&mut self, first_ev: &Event) -> Result<Yaml, ScanError> {
        let mut ev = try!(self.parse());
        let mut vec = Vec::new();
        while ev != Event::SequenceEnd {
            let entry = try!(self.load_node(&ev));
            vec.push(entry);

            // next event
            ev = try!(self.parse());
        }
        Ok(Yaml::Array(vec))
    }

    fn state_machine(&mut self) -> ParseResult {
        let next_tok = self.peek();
        println!("cur_state {:?}, next tok: {:?}", self.state, next_tok);
        match self.state {
            State::StreamStart => self.stream_start(),
            State::ImplicitDocumentStart => self.document_start(true),
            State::DocumentStart => self.document_start(false),
            State::DocumentContent => self.document_content(),

            State::BlockNode => self.parse_node(true, false),
            State::BlockNodeOrIndentlessSequence => self.parse_node(true, true),
            State::FlowNode => self.parse_node(false, false),

            State::BlockMappingFirstKey => self.block_mapping_key(true),
            State::BlockMappingKey => self.block_mapping_key(false),
            State::BlockMappingValue => self.block_mapping_value(),

            State::BlockSequenceFirstEntry => self.block_sequence_entry(true),
            State::BlockSequenceEntry => self.block_sequence_entry(false),

            State::FlowSequenceFirstEntry => self.flow_sequence_entry(true),
            State::FlowSequenceEntry => self.flow_sequence_entry(false),

            _ => unimplemented!()
        }
    }

    fn stream_start(&mut self) -> ParseResult {
        let tok = try!(self.peek());

        match tok.1 {
            TokenType::StreamStartToken(_) => {
                self.state = State::ImplicitDocumentStart;
                self.skip();
                Ok(Event::StreamStart)
            },
            _ => return Err(ScanError::new(tok.0,
                    "did not find expected <stream-start>")),
        }
    }

    fn document_start(&mut self, implicit: bool) -> ParseResult {
        let mut tok = try!(self.peek());
        if !implicit {
            loop {
                match tok.1 {
                    TokenType::DocumentEndToken => {
                        self.skip();
                        tok = try!(self.peek());
                    },
                    _ => break
                }
            }
        }

        match tok.1 {
            TokenType::StreamEndToken => {
                self.state = State::End;
                self.skip();
                return Ok(Event::StreamEnd);
            },
            TokenType::VersionDirectiveToken
                | TokenType::TagDirectiveToken
                | TokenType::DocumentStartToken => {
                    // explicit document
                    self._explict_document_start()
                },
            _ if implicit => {
                self.push_state(State::DocumentEnd);
                self.state = State::BlockNode;
                Ok(Event::DocumentStart)
            },
            _ => {
                // explicit document
                self._explict_document_start()
            }
        }
    }

    fn _explict_document_start(&mut self) -> ParseResult {
        let tok = try!(self.peek());
        if tok.1 != TokenType::DocumentStartToken {
            return Err(ScanError::new(tok.0, "did not find expected <document start>"));
        }
        self.push_state(State::DocumentEnd);
        self.state = State::DocumentContent;
        self.skip();
        Ok(Event::DocumentStart)
    }

    fn document_content(&mut self) -> ParseResult {
        let tok = try!(self.peek());
        match tok.1 {
            TokenType::VersionDirectiveToken 
                |TokenType::TagDirectiveToken
                |TokenType::DocumentStartToken
                |TokenType::DocumentEndToken
                |TokenType::StreamEndToken => {
                    self.pop_state();
                    // empty scalar
                    Ok(Event::empty_scalar())
                },
            _ => {
                self.parse_node(true, false)
            }
        }
    }

    fn parse_node(&mut self, block: bool, indentless_sequence: bool) -> ParseResult {
        let tok = try!(self.peek());
        match tok.1 {
            TokenType::AliasToken => unimplemented!(),
            TokenType::AnchorToken => unimplemented!(),
            TokenType::BlockEntryToken if indentless_sequence => {
                self.state = State::IndentlessSequenceEntry;
                Ok(Event::SequenceStart)
            },
            TokenType::ScalarToken(_, v) => {
                self.pop_state();
                self.skip();
                Ok(Event::Scalar(v))
            },
            TokenType::FlowSequenceStartToken => {
                self.state = State::FlowSequenceFirstEntry;
                Ok(Event::SequenceStart)
            },
            TokenType::FlowMappingStartToken => {
                self.state = State::FlowMappingFirstKey;
                Ok(Event::MappingStart)
            },
            TokenType::BlockSequenceStartToken if block => {
                self.state = State::BlockSequenceFirstEntry;
                Ok(Event::SequenceStart)
            },
            TokenType::BlockMappingStartToken if block => {
                self.state = State::BlockMappingFirstKey;
                Ok(Event::MappingStart)
            },
            _ => { unimplemented!(); }
        }
    }

    fn block_mapping_key(&mut self, first: bool) -> ParseResult {
        // skip BlockMappingStartToken
        if first {
            let _ = try!(self.peek());
            //self.marks.push(tok.0);
            self.skip();
        }
        let tok = try!(self.peek());
        match tok.1 {
            TokenType::KeyToken => {
                self.skip();
                let tok = try!(self.peek());
                match tok.1 {
                    TokenType::KeyToken | TokenType::ValueToken | TokenType::BlockEndToken
                        => {
                            self.state = State::BlockMappingValue;
                            // empty scalar
                            Ok(Event::empty_scalar())
                        }
                    _ => {
                        self.push_state(State::BlockMappingValue);
                        self.parse_node(true, true)
                    }
                }
            },
            TokenType::BlockEndToken => {
                self.pop_state();
                self.skip();
                Ok(Event::MappingEnd)
            },
            _ => {
                Err(ScanError::new(tok.0, "while parsing a block mapping, did not find expected key"))
            }
        }
    }

    fn block_mapping_value(&mut self) -> ParseResult {
            let tok = try!(self.peek());
            match tok.1 {
                TokenType::ValueToken => {
                    self.skip();
                    let tok = try!(self.peek());
                    match tok.1 {
                        TokenType::KeyToken | TokenType::ValueToken | TokenType::BlockEndToken
                            => {
                                self.state = State::BlockMappingValue;
                                // empty scalar
                                Ok(Event::empty_scalar())
                            }
                        _ => {
                            self.push_state(State::BlockMappingKey);
                            self.parse_node(true, true)
                        }
                    }
                },
                _ => {
                    self.state = State::BlockMappingKey;
                    // empty scalar
                    Ok(Event::empty_scalar())
                }
            }
    }

    fn flow_sequence_entry(&mut self, first: bool) -> ParseResult {
        // skip FlowMappingStartToken
        if first {
            let _ = try!(self.peek());
            //self.marks.push(tok.0);
            self.skip();
        }
        let mut tok = try!(self.peek());
        match tok.1 {
            TokenType::FlowSequenceEndToken => {
                self.pop_state();
                self.skip();
                return Ok(Event::SequenceEnd);
            },
            TokenType::FlowEntryToken if !first => {
                self.skip();
                tok = try!(self.peek());
            },
            _ if !first => {
                return Err(ScanError::new(tok.0,
                        "while parsing a flow sequence, expectd ',' or ']'"));
            }
            _ => { /* next */ }
        }
        match tok.1 {
            TokenType::FlowMappingEndToken => {
                self.pop_state();
                self.skip();
                Ok(Event::SequenceEnd)
            },
            TokenType::KeyToken => {
                self.state = State::FlowSequenceEntryMappingKey;
                self.skip();
                Ok(Event::MappingStart)
            }
            _ => {
                self.push_state(State::FlowSequenceEntry);
                self.parse_node(false, false)
            }
        }
    }

    fn block_sequence_entry(&mut self, first: bool) -> ParseResult {
        // BLOCK-SEQUENCE-START
        if first {
            let _ = try!(self.peek());
            //self.marks.push(tok.0);
            self.skip();
        }
        let mut tok = try!(self.peek());
        match tok.1 {
            TokenType::BlockEndToken => {
                self.pop_state();
                self.skip();
                Ok(Event::SequenceEnd)
            },
            TokenType::BlockEntryToken => {
                self.skip();
                tok = try!(self.peek());
                match tok.1 {
                    TokenType::BlockEntryToken | TokenType::BlockEndToken => {
                        self.state = State::BlockSequenceEntry;
                        Ok(Event::empty_scalar())
                    },
                    _ => {
                        self.push_state(State::BlockSequenceEntry);
                        self.parse_node(true, false)
                    }
                }
            },
            _ => {
                Err(ScanError::new(tok.0,
                        "while parsing a block collection, did not find expected '-' indicator"))
            }
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parser() {
        let s: String = "
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
        let mut parser = Parser::new(s.chars());
        let out = parser.load().unwrap();
        println!("DOC {:?}", out);
    }
}

