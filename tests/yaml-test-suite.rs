use std::fs::{self, DirEntry};

use libtest_mimic::{run_tests, Arguments, Outcome, Test};

use yaml_rust::{
    parser::{Event, EventReceiver, Parser},
    scanner::{TScalarStyle, TokenType},
    yaml, ScanError, Yaml, YamlLoader,
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

struct YamlTest {
    yaml_visual: String,
    yaml: String,
    expected_events: String,
    expected_error: bool,
    is_xfail: bool,
}

fn main() -> Result<()> {
    let mut arguments = Arguments::from_args();
    if arguments.num_threads.is_none() {
        arguments.num_threads = Some(1);
    }
    let tests: Vec<Vec<_>> = std::fs::read_dir("tests/yaml-test-suite/src")?
        .map(|entry| -> Result<_> {
            let entry = entry?;
            let tests = load_tests_from_file(&entry)?;
            Ok(tests)
        })
        .collect::<Result<_>>()?;
    let mut tests: Vec<_> = tests.into_iter().flatten().collect();
    tests.sort_by_key(|t| t.name.clone());

    let missing_xfails: Vec<_> = EXPECTED_FAILURES
        .iter()
        .filter(|&&test| !tests.iter().any(|t| t.name == test))
        .collect();
    if !missing_xfails.is_empty() {
        panic!(
            "The following EXPECTED_FAILURES not found during discovery: {:?}",
            missing_xfails
        );
    }

    run_tests(&arguments, tests, run_yaml_test).exit();
}

fn run_yaml_test(test: &Test<YamlTest>) -> Outcome {
    let desc = &test.data;
    let actual_events = parse_to_events(&desc.yaml);
    let events_diff = actual_events.map(|events| events_differ(events, &desc.expected_events));
    let mut error_text = match (events_diff, desc.expected_error) {
        (Ok(_), true) => Some("no error when expected".into()),
        (Err(_), true) => None,
        (Err(e), false) => Some(format!("unexpected error {:?}", e)),
        (Ok(Some(diff)), false) => Some(format!("events differ: {}", diff)),
        (Ok(None), false) => None,
    };
    if let Some(text) = &mut error_text {
        use std::fmt::Write;
        let _ = write!(text, "\n### Input:\n{}\n### End", desc.yaml_visual);
    }
    match (error_text, desc.is_xfail) {
        (None, false) => Outcome::Passed,
        (Some(text), false) => Outcome::Failed { msg: Some(text) },
        (Some(_), true) => Outcome::Ignored,
        (None, true) => Outcome::Failed {
            msg: Some("expected to fail but passes".into()),
        },
    }
}

fn load_tests_from_file(entry: &DirEntry) -> Result<Vec<Test<YamlTest>>> {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let test_name = file_name
        .strip_suffix(".yaml")
        .ok_or("unexpected filename")?;
    let tests = YamlLoader::load_from_str(&fs::read_to_string(&entry.path())?)?;
    let tests = tests[0].as_vec().ok_or("no test list found in file")?;

    let mut result = vec![];
    let mut current_test = yaml::Hash::new();
    for (idx, test_data) in tests.iter().enumerate() {
        let name = if tests.len() > 1 {
            format!("{}-{:02}", test_name, idx)
        } else {
            test_name.to_string()
        };
        let is_xfail = EXPECTED_FAILURES.contains(&name.as_str());

        // Test fields except `fail` are "inherited"
        let test_data = test_data.as_hash().unwrap();
        current_test.remove(&Yaml::String("fail".into()));
        for (key, value) in test_data.clone() {
            current_test.insert(key, value);
        }

        let current_test = Yaml::Hash(current_test.clone()); // Much better indexing

        if current_test["skip"] != Yaml::BadValue {
            continue;
        }

        result.push(Test {
            name,
            kind: String::new(),
            is_ignored: false,
            is_bench: false,
            data: YamlTest {
                yaml_visual: current_test["yaml"].as_str().unwrap().to_string(),
                yaml: visual_to_raw(current_test["yaml"].as_str().unwrap()),
                expected_events: visual_to_raw(current_test["tree"].as_str().unwrap()),
                expected_error: current_test["fail"].as_bool() == Some(true),
                is_xfail,
            },
        });
    }
    Ok(result)
}

fn parse_to_events(source: &str) -> Result<Vec<String>, ScanError> {
    let mut reporter = EventReporter::new();
    Parser::new(source.chars()).load(&mut reporter, true)?;
    Ok(reporter.events)
}

struct EventReporter {
    events: Vec<String>,
}

impl EventReporter {
    fn new() -> Self {
        Self { events: vec![] }
    }
}

impl EventReceiver for EventReporter {
    fn on_event(&mut self, ev: Event) {
        let line: String = match ev {
            Event::StreamStart => "+STR".into(),
            Event::StreamEnd => "-STR".into(),

            Event::DocumentStart => "+DOC".into(),
            Event::DocumentEnd => "-DOC".into(),

            Event::SequenceStart(idx) => format!("+SEQ{}", format_index(idx)),
            Event::SequenceEnd => "-SEQ".into(),

            Event::MappingStart(idx) => format!("+MAP{}", format_index(idx)),
            Event::MappingEnd => "-MAP".into(),

            Event::Scalar(ref text, style, idx, ref tag) => {
                let kind = match style {
                    TScalarStyle::Plain => ":",
                    TScalarStyle::SingleQuoted => "'",
                    TScalarStyle::DoubleQuoted => r#"""#,
                    TScalarStyle::Literal => "|",
                    TScalarStyle::Foled => ">",
                    TScalarStyle::Any => unreachable!(),
                };
                format!(
                    "=VAL{}{} {}{}",
                    format_index(idx),
                    format_tag(tag),
                    kind,
                    escape_text(text)
                )
            }
            Event::Alias(idx) => format!("=ALI *{}", idx),
            Event::Nothing => return,
        };
        self.events.push(line);
    }
}

fn format_index(idx: usize) -> String {
    if idx > 0 {
        format!(" &{}", idx)
    } else {
        "".into()
    }
}

fn escape_text(text: &str) -> String {
    let mut text = text.to_owned();
    for (ch, replacement) in [
        ('\\', r#"\\"#),
        ('\n', "\\n"),
        ('\r', "\\r"),
        ('\x08', "\\b"),
        ('\t', "\\t"),
    ] {
        text = text.replace(ch, replacement);
    }
    text
}

fn format_tag(tag: &Option<TokenType>) -> String {
    if let Some(TokenType::Tag(ns, tag)) = tag {
        let ns = match ns.as_str() {
            "!!" => "tag:yaml.org,2002:", // Wrong if this ns is overridden
            other => other,
        };
        format!(" <{}{}>", ns, tag)
    } else {
        "".into()
    }
}

fn events_differ(actual: Vec<String>, expected: &str) -> Option<String> {
    let actual = actual.iter().map(Some).chain(std::iter::repeat(None));
    let expected = expected_events(expected);
    let expected = expected.iter().map(Some).chain(std::iter::repeat(None));
    for (idx, (act, exp)) in actual.zip(expected).enumerate() {
        return match (act, exp) {
            (Some(act), Some(exp)) => {
                if act == exp {
                    continue;
                } else {
                    Some(format!(
                        "line {} differs: expected `{}`, found `{}`",
                        idx, exp, act
                    ))
                }
            }
            (Some(a), None) => Some(format!("extra actual line: {:?}", a)),
            (None, Some(e)) => Some(format!("extra expected line: {:?}", e)),
            (None, None) => None,
        };
    }
    unreachable!()
}

/// Convert the snippets from "visual" to "actual" representation
fn visual_to_raw(yaml: &str) -> String {
    let mut yaml = yaml.to_owned();
    for (pat, replacement) in [
        ("␣", " "),
        ("»", "\t"),
        ("—", ""), // Tab line continuation ——»
        ("←", "\r"),
        ("⇔", "\u{FEFF}"),
        ("↵", ""), // Trailing newline marker
        ("∎\n", ""),
    ] {
        yaml = yaml.replace(pat, replacement);
    }
    yaml
}

/// Adapt the expectations to the yaml-rust reasonable limitations
///
/// Drop information on node styles (flow/block) and anchor names.
/// Both are things that can be omitted according to spec.
fn expected_events(expected_tree: &str) -> Vec<String> {
    let mut anchors = vec![];
    expected_tree
        .split('\n')
        .map(|s| s.trim_start().to_owned())
        .filter(|s| !s.is_empty())
        .map(|mut s| {
            // Anchor name-to-number conversion
            if let Some(start) = s.find('&') {
                if s[..start].find(':').is_none() {
                    let len = s[start..].find(' ').unwrap_or(s[start..].len());
                    anchors.push(s[start + 1..start + len].to_owned());
                    s = s.replace(&s[start..start + len], &format!("&{}", anchors.len()));
                }
            }
            // Alias nodes name-to-number
            if s.starts_with("=ALI") {
                let start = s.find('*').unwrap();
                let name = &s[start + 1..];
                let idx = anchors
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| v == &name)
                    .last()
                    .unwrap()
                    .0;
                s = s.replace(&s[start..], &format!("*{}", idx + 1));
            }
            // Dropping style information
            match &*s {
                "+DOC ---" => "+DOC".into(),
                "-DOC ..." => "-DOC".into(),
                s if s.starts_with("+SEQ []") => s.replacen("+SEQ []", "+SEQ", 1),
                s if s.starts_with("+MAP {}") => s.replacen("+MAP {}", "+MAP", 1),
                "=VAL :" => "=VAL :~".into(), // FIXME: known bug
                s => s.into(),
            }
        })
        .collect()
}

#[rustfmt::skip]
static EXPECTED_FAILURES: &[&str] = &[
    // These seem to be API limited (not enough information on the event stream level)
    // No tag available for SEQ and MAP
    "2XXW",
    "35KP",
    "57H4",
    "6JWB",
    "735Y",
    "9KAX",
    "BU8L",
    "C4HZ",
    "EHF6",
    "J7PZ",
    "UGM3",
    // Cannot resolve tag namespaces
    "5TYM",
    "6CK3",
    "6WLZ",
    "9WXW",
    "CC74",
    "U3C3",
    "Z9M4",
    "P76L", // overriding the `!!` namespace!

    // These seem to be plain bugs
    // Leading TAB in literal scalars
    "96NN-00",
    "96NN-01",
    "R4YG",
    "Y79Y-01",
    // TAB as start of plain scalar instead of whitespace
    "6BCT",
    "6CA3",
    "A2M4",
    "DK95-00",
    "Q5MG",
    "Y79Y-06",
    "4EJS", // unexpected pass
    "Y79Y-03", // unexpected pass
    "Y79Y-04", // unexpected pass
    "Y79Y-05", // unexpected pass
    "Y79Y-10",
    // TABs in whitespace-only lines
    "DK95-03",
    "DK95-04",
    // TABs after marker ? or : (space required?)
    "Y79Y-07",
    "Y79Y-08",
    "Y79Y-09",
    // Other TABs
    "DK95-01", // in double-quoted scalar
    // Empty key in flow mappings
    "CFD4",
    // Document with no nodes and document end
    "HWV9",
    "QT73",
    // Unusual characters in anchors/aliases
    "8XYN", // emoji!!
    "W5VH", // :@*!$"<foo>:
    // Flow mapping colon on next line / multiline key in flow mapping
    "4MUZ-00",
    "4MUZ-01",
    "4MUZ-02",
    "5MUD",
    "9SA2",
    "K3WX",
    "NJ66",
    "UT92",
    "VJP3-01",
    // Bare document after end marker
    "7Z25",
    "M7A3",
    // Scalar marker on document start line
    "DK3J",
    "FP8R",
    // Comments on nonempty lines need leading space
    "9JBA",
    "CVW2",
    "MUS6-00",
    "SU5Z",
    "X4QW",
    // Directives (various)
    "9HCY", // Directive after content
    "EB22", // Directive after content
    "MUS6-01", // no document end marker?
    "QLJ7", // TAG directives should not be inherited between documents
    "RHX7", // no document end marker
    "SF5V", // duplicate directive
    "W4TN", // scalar confused as directive
    // Losing trailing newline
    "JEF9-02",
    "L24T-01",
    // Dashes in flow sequence (should be forbidden)
    "G5U8",
    "YJV2",
    // Misc
    "9MMW", // Mapping key in implicit mapping in flow sequence(!)
    "G9HC", // Anchor indent problem(?)
    "H7J7", // Anchor indent / linebreak problem?
    "3UYS", // Escaped /
    "HRE5", // Escaped ' in double-quoted (should not work)
    "QB6E", // Indent for multiline double-quoted scalar
    "S98Z", // Block scalar and indent problems?
    "U99R", // Comma is not allowed in tags
    "WZ62", // Empty content
];
