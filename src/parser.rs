use scanner::*;
// use yaml::*;

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
    Scalar(String, TScalarStyle),
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd
}

impl Event {
    fn empty_scalar() -> Event {
        // a null scalar
        Event::Scalar("~".to_string(), TScalarStyle::Plain)
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

pub trait EventReceiver {
    fn on_event(&mut self, ev: &Event);
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
            match self.scanner.get_error() {
                None =>
                return Err(ScanError::new(self.scanner.mark(), 
                      "unexpected eof")),
                Some(e) => return Err(e),
            }
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

    fn parse<R: EventReceiver>(&mut self, recv: &mut R)
        -> ParseResult {
        if self.scanner.stream_ended()
            || self.state == State::End {
            return Ok(Event::StreamEnd);
        }
        let ev = try!(self.state_machine());
        // println!("EV {:?}", ev);
        recv.on_event(&ev);
        Ok(ev)
    }

    pub fn load<R: EventReceiver>(&mut self, recv: &mut R, multi: bool)
        -> Result<(), ScanError> {
        if !self.scanner.stream_started() {
            let ev = try!(self.parse(recv));
            assert_eq!(ev, Event::StreamStart);
        }

        if self.scanner.stream_ended() {
            // XXX has parsed?
            recv.on_event(&Event::StreamEnd);
            return Ok(());
        }
        loop {
            let ev = try!(self.parse(recv));
            if ev == Event::StreamEnd {
                recv.on_event(&Event::StreamEnd);
                return Ok(());
            }
            try!(self.load_document(&ev, recv));
            if !multi {
                break;
            }
        }
        Ok(())
    }

    fn load_document<R: EventReceiver>(&mut self, first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        assert_eq!(first_ev, &Event::DocumentStart);

        let ev = try!(self.parse(recv));
        try!(self.load_node(&ev, recv));

        // DOCUMENT-END is expected.
        let ev = try!(self.parse(recv));
        assert_eq!(ev, Event::DocumentEnd);

        Ok(())
    }

    fn load_node<R: EventReceiver>(&mut self, first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        match *first_ev {
            Event::Alias => { unimplemented!() },
            Event::Scalar(_, _) => {
                Ok(())
            },
            Event::SequenceStart => {
                self.load_sequence(first_ev, recv)
            },
            Event::MappingStart => {
                self.load_mapping(first_ev, recv)
            },
            _ => { println!("UNREACHABLE EVENT: {:?}", first_ev);
                unreachable!(); }
        }
    }

    fn load_mapping<R: EventReceiver>(&mut self, _first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        let mut ev = try!(self.parse(recv));
        while ev != Event::MappingEnd {
            // key
            try!(self.load_node(&ev, recv));

            // value
            ev = try!(self.parse(recv));
            try!(self.load_node(&ev, recv));

            // next event
            ev = try!(self.parse(recv));
        }
        Ok(())
    }

    fn load_sequence<R: EventReceiver>(&mut self, _first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        let mut ev = try!(self.parse(recv));
        while ev != Event::SequenceEnd {
            try!(self.load_node(&ev, recv));

            // next event
            ev = try!(self.parse(recv));
        }
        Ok(())
    }

    fn state_machine(&mut self) -> ParseResult {
        let next_tok = try!(self.peek());
        println!("cur_state {:?}, next tok: {:?}", self.state, next_tok);
        match self.state {
            State::StreamStart => self.stream_start(),

            State::ImplicitDocumentStart => self.document_start(true),
            State::DocumentStart => self.document_start(false),
            State::DocumentContent => self.document_content(),
            State::DocumentEnd => self.document_end(),

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

            State::FlowMappingFirstKey => self.flow_mapping_key(true),
            State::FlowMappingKey => self.flow_mapping_key(false),
            State::FlowMappingValue => self.flow_mapping_value(false),

            State::IndentlessSequenceEntry => self.indentless_sequence_entry(),

            State::FlowSequenceEntryMappingKey => self.flow_sequence_entry_mapping_key(),
            State::FlowSequenceEntryMappingValue => self.flow_sequence_entry_mapping_value(),
            State::FlowSequenceEntryMappingEnd => self.flow_sequence_entry_mapping_end(),

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
            TokenType::VersionDirectiveToken(..)
                | TokenType::TagDirectiveToken
                | TokenType::DocumentStartToken => {
                    // explicit document
                    self._explict_document_start()
                },
            _ if implicit => {
                try!(self.parser_process_directives());
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

    fn parser_process_directives(&mut self) -> Result<(), ScanError> {
        loop {
            let tok = try!(self.peek());
            match tok.1 {
                TokenType::VersionDirectiveToken(_, _) => {
                    // XXX parsing with warning according to spec
                    //if major != 1 || minor > 2 {
                    //    return Err(ScanError::new(tok.0,
                    //        "found incompatible YAML document"));
                    //}
                },
                TokenType::TagDirectiveToken => {
                    unimplemented!();
                },
                _ => break
            }
            self.skip();
        }
        // TODO tag directive
        Ok(())
    }

    fn _explict_document_start(&mut self) -> ParseResult {
        try!(self.parser_process_directives());
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
            TokenType::VersionDirectiveToken(..)
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

    fn document_end(&mut self) -> ParseResult {
        let mut _implicit = true;
        let tok = try!(self.peek());
        let _start_mark = tok.0;

        match tok.1 {
            TokenType::DocumentEndToken => {
                self.skip();
                _implicit = false;
            }
            _ => {}
        }

        // TODO tag handling
        self.state = State::DocumentStart;
        Ok(Event::DocumentEnd)
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
            TokenType::ScalarToken(style, v) => {
                self.pop_state();
                self.skip();
                Ok(Event::Scalar(v, style))
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
            _ => { Err(ScanError::new(tok.0, "while parsing a node, did not find expected node content")) }
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
                    TokenType::KeyToken
                        | TokenType::ValueToken
                        | TokenType::BlockEndToken
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
            // XXX(chenyh): libyaml failed to parse spec 1.2, ex8.18
            TokenType::ValueToken => {
                self.state = State::BlockMappingValue;
                Ok(Event::empty_scalar())
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
                                self.state = State::BlockMappingKey;
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

    fn flow_mapping_key(&mut self, first: bool) -> ParseResult {
        if first {
            let _ = try!(self.peek());
            self.skip();
        }
        let mut tok = try!(self.peek());

        if tok.1 != TokenType::FlowMappingEndToken {
            if !first {
                if tok.1 == TokenType::FlowEntryToken {
                    self.skip();
                    tok = try!(self.peek());
                } else {
                    return Err(ScanError::new(tok.0,
                        "while parsing a flow mapping, did not find expected ',' or '}'"));
                }
            }

            if tok.1 == TokenType::KeyToken {
                self.skip();
                tok = try!(self.peek());
                match tok.1 {
                    TokenType::ValueToken
                        | TokenType::FlowEntryToken
                        | TokenType::FlowMappingEndToken => {
                        self.state = State::FlowMappingValue;
                        return Ok(Event::empty_scalar());
                    },
                    _ => {
                        self.push_state(State::FlowMappingValue);
                        return self.parse_node(false, false);
                    }
                }
            // XXX libyaml fail ex 7.3, empty key
            } else if tok.1 == TokenType::ValueToken {
                self.state = State::FlowMappingValue;
                return Ok(Event::empty_scalar());
            } else if tok.1 != TokenType::FlowMappingEndToken {
                self.push_state(State::FlowMappingEmptyValue);
                return self.parse_node(false, false);
            }
        }

        self.pop_state();
        self.skip();
        Ok(Event::MappingEnd)
    }

    fn flow_mapping_value(&mut self, empty: bool) -> ParseResult {
        let tok = try!(self.peek());
        if empty {
            self.state = State::FlowMappingKey;
            return Ok(Event::empty_scalar());
        }

        if tok.1 == TokenType::ValueToken {
            self.skip();
            let tok = try!(self.peek());
            match tok.1 {
                TokenType::FlowEntryToken 
                    | TokenType::FlowMappingEndToken => { },
                _ => {
                        self.push_state(State::FlowMappingKey);
                        return self.parse_node(false, false);
                }
            }
        }

        self.state = State::FlowMappingKey;
        Ok(Event::empty_scalar())
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
            TokenType::FlowSequenceEndToken => {
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

    fn indentless_sequence_entry(&mut self) -> ParseResult {
        let mut tok = try!(self.peek());
        if tok.1 != TokenType::BlockEntryToken {
            self.pop_state();
            return Ok(Event::SequenceEnd);
        }

        self.skip();
        tok = try!(self.peek());
        match tok.1 {
            TokenType::BlockEntryToken
                | TokenType::KeyToken
                | TokenType::ValueToken
                | TokenType::BlockEndToken => {
                self.state = State::IndentlessSequenceEntry;
                Ok(Event::empty_scalar())
            },
            _ => {
                self.push_state(State::IndentlessSequenceEntry);
                self.parse_node(true, false)
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

    fn flow_sequence_entry_mapping_key(&mut self) -> ParseResult {
        let tok = try!(self.peek());

        match tok.1 {
            TokenType::ValueToken
                | TokenType::FlowEntryToken
                | TokenType::FlowSequenceEndToken => {
                    self.skip();
                    self.state = State::FlowSequenceEntryMappingValue;
                    Ok(Event::empty_scalar())
            },
            _ => {
                self.push_state(State::FlowSequenceEntryMappingValue);
                self.parse_node(false, false)
            }
        }
    }

    fn flow_sequence_entry_mapping_value(&mut self) -> ParseResult {
        let tok = try!(self.peek());

        match tok.1 {
            TokenType::ValueToken => {
                    self.skip();
                    let tok = try!(self.peek());
                    self.state = State::FlowSequenceEntryMappingValue;
                    match tok.1 {
                        TokenType::FlowEntryToken
                            | TokenType::FlowSequenceEndToken => {
                                self.state = State::FlowSequenceEntryMappingEnd;
                                Ok(Event::empty_scalar())
                        },
                        _ => {
                            self.push_state(State::FlowSequenceEntryMappingEnd);
                            self.parse_node(false, false)
                        }
                    }
            },
            _ => {
                self.state = State::FlowSequenceEntryMappingEnd;
                Ok(Event::empty_scalar())
            }
        }
    }

    fn flow_sequence_entry_mapping_end(&mut self) -> ParseResult {
        self.state = State::FlowSequenceEntry;
        Ok(Event::MappingEnd)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}

