use scanner::*;
use std::collections::HashMap;
// use yaml::*;

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
enum State {
    StreamStart,
    ImplicitDocumentStart,
    DocumentStart,
    DocumentContent,
    DocumentEnd,
    BlockNode,
    // BlockNodeOrIndentlessSequence,
    // FlowNode,
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

/// `Event` is used with the low-level event base parsing API,
/// see `EventReceiver` trait.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Event {
    /// Reserved for internal use
    Nothing,
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    /// Refer to an anchor ID
    Alias(usize),
    /// Value, style, anchor_id, tag
    Scalar(String, TScalarStyle, usize, Option<TokenType>),
    /// Anchor ID
    SequenceStart(usize),
    SequenceEnd,
    /// Anchor ID
    MappingStart(usize),
    MappingEnd
}

impl Event {
    fn empty_scalar() -> Event {
        // a null scalar
        Event::Scalar("~".to_owned(), TScalarStyle::Plain, 0, None)
    }

    fn empty_scalar_with_anchor(anchor: usize, tag: Option<TokenType>) -> Event {
        Event::Scalar("".to_owned(), TScalarStyle::Plain, anchor, tag)
    }
}

#[derive(Debug)]
pub struct Parser<T> {
    scanner: Scanner<T>,
    states: Vec<State>,
    state: State,
    marks: Vec<Marker>,
    token: Option<Token>,
    anchors: HashMap<String, usize>,
    anchor_id: usize,
}


pub trait EventReceiver {
    fn on_event(&mut self, ev: &Event);
}


pub trait MarkedEventReceiver {
    fn on_event(&mut self, ev: &Event, _mark: Marker);
}

impl<R: EventReceiver> MarkedEventReceiver for R {
    fn on_event(&mut self, ev: &Event, _mark: Marker) {
        self.on_event(ev)
    }
}



pub type ParseResult = Result<(Event, Marker), ScanError>;

impl<T: Iterator<Item=char>> Parser<T> {
    pub fn new(src: T) -> Parser<T> {
        Parser {
            scanner: Scanner::new(src),
            states: Vec::new(),
            state: State::StreamStart,
            marks: Vec::new(),
            token: None,

            anchors: HashMap::new(),
            // valid anchor_id starts from 1
            anchor_id: 1,
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

    fn parse<R: MarkedEventReceiver>(&mut self, recv: &mut R)
        -> Result<Event, ScanError> {
        if self.state == State::End {
            return Ok(Event::StreamEnd);
        }
        let (ev, mark) = try!(self.state_machine());
        // println!("EV {:?}", ev);
        recv.on_event(&ev, mark);
        Ok(ev)
    }

    pub fn load<R: MarkedEventReceiver>(&mut self, recv: &mut R, multi: bool)
        -> Result<(), ScanError> {
        if !self.scanner.stream_started() {
            let ev = try!(self.parse(recv));
            assert_eq!(ev, Event::StreamStart);
        }

        if self.scanner.stream_ended() {
            // XXX has parsed?
            recv.on_event(&Event::StreamEnd, self.scanner.mark());
            return Ok(());
        }
        loop {
            let ev = try!(self.parse(recv));
            if ev == Event::StreamEnd {
                recv.on_event(&Event::StreamEnd, self.scanner.mark());
                return Ok(());
            }
            // clear anchors before a new document
            self.anchors.clear();
            try!(self.load_document(&ev, recv));
            if !multi {
                break;
            }
        }
        Ok(())
    }

    fn load_document<R: MarkedEventReceiver>(&mut self, first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        assert_eq!(first_ev, &Event::DocumentStart);

        let ev = try!(self.parse(recv));
        try!(self.load_node(&ev, recv));

        // DOCUMENT-END is expected.
        let ev = try!(self.parse(recv));
        assert_eq!(ev, Event::DocumentEnd);

        Ok(())
    }

    fn load_node<R: MarkedEventReceiver>(&mut self, first_ev: &Event, recv: &mut R)
        -> Result<(), ScanError> {
        match *first_ev {
            Event::Alias(..) | Event::Scalar(..) => {
                Ok(())
            },
            Event::SequenceStart(_) => {
                self.load_sequence(first_ev, recv)
            },
            Event::MappingStart(_) => {
                self.load_mapping(first_ev, recv)
            },
            _ => { println!("UNREACHABLE EVENT: {:?}", first_ev);
                unreachable!(); }
        }
    }

    fn load_mapping<R: MarkedEventReceiver>(&mut self, _first_ev: &Event, recv: &mut R)
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

    fn load_sequence<R: MarkedEventReceiver>(&mut self, _first_ev: &Event, recv: &mut R)
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
        // let next_tok = try!(self.peek());
        // println!("cur_state {:?}, next tok: {:?}", self.state, next_tok);
        match self.state {
            State::StreamStart => self.stream_start(),

            State::ImplicitDocumentStart => self.document_start(true),
            State::DocumentStart => self.document_start(false),
            State::DocumentContent => self.document_content(),
            State::DocumentEnd => self.document_end(),

            State::BlockNode => self.parse_node(true, false),
            // State::BlockNodeOrIndentlessSequence => self.parse_node(true, true),
            // State::FlowNode => self.parse_node(false, false),

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
            State::FlowMappingEmptyValue => self.flow_mapping_value(true),

            /* impossible */
            State::End => unreachable!(),
        }
    }

    fn stream_start(&mut self) -> ParseResult {
        let tok = try!(self.peek());

        match tok.1 {
            TokenType::StreamStart(_) => {
                self.state = State::ImplicitDocumentStart;
                self.skip();
                Ok((Event::StreamStart, tok.0))
            },
            _ => Err(ScanError::new(tok.0,
                    "did not find expected <stream-start>")),
        }
    }

    fn document_start(&mut self, implicit: bool) -> ParseResult {
        let mut tok = try!(self.peek());
        if !implicit {
            while let TokenType::DocumentEnd = tok.1 {
                self.skip();
                tok = try!(self.peek());
            }
        }

        match tok.1 {
            TokenType::StreamEnd => {
                self.state = State::End;
                self.skip();
                Ok((Event::StreamEnd, tok.0))
            },
            TokenType::VersionDirective(..)
                | TokenType::TagDirective(..)
                | TokenType::DocumentStart => {
                    // explicit document
                    self._explict_document_start()
                },
            _ if implicit => {
                try!(self.parser_process_directives());
                self.push_state(State::DocumentEnd);
                self.state = State::BlockNode;
                Ok((Event::DocumentStart, tok.0))
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
                TokenType::VersionDirective(_, _) => {
                    // XXX parsing with warning according to spec
                    //if major != 1 || minor > 2 {
                    //    return Err(ScanError::new(tok.0,
                    //        "found incompatible YAML document"));
                    //}
                },
                TokenType::TagDirective(..) => {
                    // TODO add tag directive
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
        if tok.1 != TokenType::DocumentStart {
            return Err(ScanError::new(tok.0, "did not find expected <document start>"));
        }
        self.push_state(State::DocumentEnd);
        self.state = State::DocumentContent;
        self.skip();
        Ok((Event::DocumentStart, tok.0))
    }

    fn document_content(&mut self) -> ParseResult {
        let tok = try!(self.peek());
        match tok.1 {
            TokenType::VersionDirective(..)
                |TokenType::TagDirective(..)
                |TokenType::DocumentStart
                |TokenType::DocumentEnd
                |TokenType::StreamEnd => {
                    self.pop_state();
                    // empty scalar
                    Ok((Event::empty_scalar(), tok.0))
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

        if let TokenType::DocumentEnd = tok.1 {
            self.skip();
            _implicit = false;
        }

        // TODO tag handling
        self.state = State::DocumentStart;
        Ok((Event::DocumentEnd, tok.0))
    }

    fn register_anchor(&mut self, name: &str, _: &Marker) -> Result<usize, ScanError> {
        // anchors can be overrided/reused
        // if self.anchors.contains_key(name) {
        //     return Err(ScanError::new(*mark,
        //         "while parsing anchor, found duplicated anchor"));
        // }
        let new_id = self.anchor_id;
        self.anchor_id += 1;
        self.anchors.insert(name.to_owned(), new_id);
        Ok(new_id)
    }

    fn parse_node(&mut self, block: bool, indentless_sequence: bool) -> ParseResult {
        let mut tok = try!(self.peek());
        let mut anchor_id = 0;
        let mut tag = None;
        match tok.1 {
            TokenType::Alias(name) => {
                self.pop_state();
                self.skip();
                match self.anchors.get(&name) {
                    None => return Err(ScanError::new(tok.0, "while parsing node, found unknown anchor")),
                    Some(id) => return Ok((Event::Alias(*id), tok.0))
                }
            },
            TokenType::Anchor(name) => {
                anchor_id = try!(self.register_anchor(&name, &tok.0));
                self.skip();
                tok = try!(self.peek());
                if let TokenType::Tag(_, _) = tok.1 {
                    tag = Some(tok.1);
                    self.skip();
                    tok = try!(self.peek());
                }
            },
            TokenType::Tag(..) => {
                tag = Some(tok.1);
                self.skip();
                tok = try!(self.peek());
                if let TokenType::Anchor(name) = tok.1 {
                    anchor_id = try!(self.register_anchor(&name, &tok.0));
                    self.skip();
                    tok = try!(self.peek());
                }
            },
            _ => {}
        }
        match tok.1 {
            TokenType::BlockEntry if indentless_sequence => {
                self.state = State::IndentlessSequenceEntry;
                Ok((Event::SequenceStart(anchor_id), tok.0))
            },
            TokenType::Scalar(style, v) => {
                self.pop_state();
                self.skip();
                Ok((Event::Scalar(v, style, anchor_id, tag), tok.0))
            },
            TokenType::FlowSequenceStart => {
                self.state = State::FlowSequenceFirstEntry;
                Ok((Event::SequenceStart(anchor_id), tok.0))
            },
            TokenType::FlowMappingStart => {
                self.state = State::FlowMappingFirstKey;
                Ok((Event::MappingStart(anchor_id), tok.0))
            },
            TokenType::BlockSequenceStart if block => {
                self.state = State::BlockSequenceFirstEntry;
                Ok((Event::SequenceStart(anchor_id), tok.0))
            },
            TokenType::BlockMappingStart if block => {
                self.state = State::BlockMappingFirstKey;
                Ok((Event::MappingStart(anchor_id), tok.0))
            },
            // ex 7.2, an empty scalar can follow a secondary tag
            _ if tag.is_some() || anchor_id > 0 => {
                self.pop_state();
                Ok((Event::empty_scalar_with_anchor(anchor_id, tag), tok.0))
            },
            _ => { Err(ScanError::new(tok.0, "while parsing a node, did not find expected node content")) }
        }
    }

    fn block_mapping_key(&mut self, first: bool) -> ParseResult {
        // skip BlockMappingStart
        if first {
            let _ = try!(self.peek());
            //self.marks.push(tok.0);
            self.skip();
        }
        let tok = try!(self.peek());
        match tok.1 {
            TokenType::Key => {
                self.skip();
                let tok = try!(self.peek());
                match tok.1 {
                    TokenType::Key
                        | TokenType::Value
                        | TokenType::BlockEnd
                        => {
                            self.state = State::BlockMappingValue;
                            // empty scalar
                            Ok((Event::empty_scalar(), tok.0))
                        }
                    _ => {
                        self.push_state(State::BlockMappingValue);
                        self.parse_node(true, true)
                    }
                }
            },
            // XXX(chenyh): libyaml failed to parse spec 1.2, ex8.18
            TokenType::Value => {
                self.state = State::BlockMappingValue;
                Ok((Event::empty_scalar(), tok.0))
            },
            TokenType::BlockEnd => {
                self.pop_state();
                self.skip();
                Ok((Event::MappingEnd, tok.0))
            },
            _ => {
                Err(ScanError::new(tok.0, "while parsing a block mapping, did not find expected key"))
            }
        }
    }

    fn block_mapping_value(&mut self) -> ParseResult {
            let tok = try!(self.peek());
            match tok.1 {
                TokenType::Value => {
                    self.skip();
                    let tok = try!(self.peek());
                    match tok.1 {
                        TokenType::Key | TokenType::Value | TokenType::BlockEnd
                            => {
                                self.state = State::BlockMappingKey;
                                // empty scalar
                                Ok((Event::empty_scalar(), tok.0))
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
                    Ok((Event::empty_scalar(), tok.0))
                }
            }
    }

    fn flow_mapping_key(&mut self, first: bool) -> ParseResult {
        if first {
            let _ = try!(self.peek());
            self.skip();
        }
        let mut tok = try!(self.peek());

        if tok.1 != TokenType::FlowMappingEnd {
            if !first {
                if tok.1 == TokenType::FlowEntry {
                    self.skip();
                    tok = try!(self.peek());
                } else {
                    return Err(ScanError::new(tok.0,
                        "while parsing a flow mapping, did not find expected ',' or '}'"));
                }
            }

            if tok.1 == TokenType::Key {
                self.skip();
                tok = try!(self.peek());
                match tok.1 {
                    TokenType::Value
                        | TokenType::FlowEntry
                        | TokenType::FlowMappingEnd => {
                        self.state = State::FlowMappingValue;
                        return Ok((Event::empty_scalar(), tok.0));
                    },
                    _ => {
                        self.push_state(State::FlowMappingValue);
                        return self.parse_node(false, false);
                    }
                }
            // XXX libyaml fail ex 7.3, empty key
            } else if tok.1 == TokenType::Value {
                self.state = State::FlowMappingValue;
                return Ok((Event::empty_scalar(), tok.0));
            } else if tok.1 != TokenType::FlowMappingEnd {
                self.push_state(State::FlowMappingEmptyValue);
                return self.parse_node(false, false);
            }
        }

        self.pop_state();
        self.skip();
        Ok((Event::MappingEnd, tok.0))
    }

    fn flow_mapping_value(&mut self, empty: bool) -> ParseResult {
        let tok = try!(self.peek());
        if empty {
            self.state = State::FlowMappingKey;
            return Ok((Event::empty_scalar(), tok.0));
        }

        if tok.1 == TokenType::Value {
            self.skip();
            let tok = try!(self.peek());
            match tok.1 {
                TokenType::FlowEntry
                    | TokenType::FlowMappingEnd => { },
                _ => {
                        self.push_state(State::FlowMappingKey);
                        return self.parse_node(false, false);
                }
            }
        }

        self.state = State::FlowMappingKey;
        Ok((Event::empty_scalar(), tok.0))
    }

    fn flow_sequence_entry(&mut self, first: bool) -> ParseResult {
        // skip FlowMappingStart
        if first {
            let _ = try!(self.peek());
            //self.marks.push(tok.0);
            self.skip();
        }
        let mut tok = try!(self.peek());
        match tok.1 {
            TokenType::FlowSequenceEnd => {
                self.pop_state();
                self.skip();
                return Ok((Event::SequenceEnd, tok.0));
            },
            TokenType::FlowEntry if !first => {
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
            TokenType::FlowSequenceEnd => {
                self.pop_state();
                self.skip();
                Ok((Event::SequenceEnd, tok.0))
            },
            TokenType::Key => {
                self.state = State::FlowSequenceEntryMappingKey;
                self.skip();
                Ok((Event::MappingStart(0), tok.0))
            }
            _ => {
                self.push_state(State::FlowSequenceEntry);
                self.parse_node(false, false)
            }
        }
    }

    fn indentless_sequence_entry(&mut self) -> ParseResult {
        let mut tok = try!(self.peek());
        if tok.1 != TokenType::BlockEntry {
            self.pop_state();
            return Ok((Event::SequenceEnd, tok.0));
        }

        self.skip();
        tok = try!(self.peek());
        match tok.1 {
            TokenType::BlockEntry
                | TokenType::Key
                | TokenType::Value
                | TokenType::BlockEnd => {
                self.state = State::IndentlessSequenceEntry;
                Ok((Event::empty_scalar(), tok.0))
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
            TokenType::BlockEnd => {
                self.pop_state();
                self.skip();
                Ok((Event::SequenceEnd, tok.0))
            },
            TokenType::BlockEntry => {
                self.skip();
                tok = try!(self.peek());
                match tok.1 {
                    TokenType::BlockEntry
                        | TokenType::BlockEnd => {
                        self.state = State::BlockSequenceEntry;
                        Ok((Event::empty_scalar(), tok.0))
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
            TokenType::Value
                | TokenType::FlowEntry
                | TokenType::FlowSequenceEnd => {
                    self.skip();
                    self.state = State::FlowSequenceEntryMappingValue;
                    Ok((Event::empty_scalar(), tok.0))
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
            TokenType::Value => {
                    self.skip();
                    let tok = try!(self.peek());
                    self.state = State::FlowSequenceEntryMappingValue;
                    match tok.1 {
                        TokenType::FlowEntry
                            | TokenType::FlowSequenceEnd => {
                                self.state = State::FlowSequenceEntryMappingEnd;
                                Ok((Event::empty_scalar(), tok.0))
                        },
                        _ => {
                            self.push_state(State::FlowSequenceEntryMappingEnd);
                            self.parse_node(false, false)
                        }
                    }
            },
            _ => {
                self.state = State::FlowSequenceEntryMappingEnd;
                Ok((Event::empty_scalar(), tok.0))
            }
        }
    }

    fn flow_sequence_entry_mapping_end(&mut self) -> ParseResult {
        self.state = State::FlowSequenceEntry;
        Ok((Event::MappingEnd, self.scanner.mark()))
    }
}
