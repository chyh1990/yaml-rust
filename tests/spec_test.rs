#![allow(dead_code)]
extern crate yaml_rust;

use yaml_rust::parser::Parser;
use yaml_rust::yaml::Yaml;


#[derive(Clone, PartialEq, PartialOrd, Debug)]
enum TestEvent {
    OnDocumentStart,
    OnDocumentEnd,
    OnSequenceStart,
    OnSequenceEnd,
    OnMapStart,
    OnMapEnd,
    OnScalar,
    OnAlias,
    OnNull,
}

fn yaml_to_test_events(root :&Yaml) -> Vec<TestEvent> {
    fn next(root: &Yaml, evs: &mut Vec<TestEvent>) {
        match *root {
            Yaml::BadValue => { panic!("unexpected BadValue"); },
            Yaml::Null => { evs.push(TestEvent::OnNull); },
            Yaml::Array(ref v) => {
                evs.push(TestEvent::OnSequenceStart);
                for e in v {
                    next(e, evs);
                }
                evs.push(TestEvent::OnSequenceEnd);
            },
            Yaml::Hash(ref v) => {
                evs.push(TestEvent::OnMapStart);
                for (k, v) in v {
                    next(k, evs);
                    next(v, evs);
                }
                evs.push(TestEvent::OnMapEnd);
            },
            _ => { evs.push(TestEvent::OnScalar); }
        }
    }
    let mut evs: Vec<TestEvent> = Vec::new();
    evs.push(TestEvent::OnDocumentStart);
    next(&root, &mut evs);
    evs.push(TestEvent::OnDocumentEnd);
    evs
}

macro_rules! assert_next {
    ($v:expr, $p:pat) => (
        match $v.next().unwrap() {
            $p => {},
            e => { panic!("unexpected event: {:?}", e); }
        }
    )
}

// auto generated from handler_spec_test.cpp
include!("specexamples.rs.inc");
include!("spec_test.rs.inc");

