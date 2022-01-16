use std::{ffi::OsStr, fs, path::Path};

use yaml_rust::{
    parser::{Event, EventReceiver, Parser},
    scanner::{TokenType, TScalarStyle},
    ScanError,
    Yaml,
    YamlLoader,
    yaml,
};

#[test]
fn yaml_test_suite() -> Result<(), Box<dyn std::error::Error>> {
    let mut error_count = 0;
    for entry in std::fs::read_dir("tests/yaml-test-suite/src")? {
        let entry = entry?;
        error_count += run_tests_from_file(&entry.path(), &entry.file_name())?;
    }
    println!("Expected errors: {}", EXPECTED_FAILURES.len());
    if error_count > 0 {
        panic!("Unexpected errors in testsuite: {}", error_count);
    }
    Ok(())
}

fn run_tests_from_file(path: impl AsRef<Path>, file_name: &OsStr) -> Result<u32, Box<dyn std::error::Error>> {
    let test_name = path.as_ref()
        .file_name().ok_or("")?
        .to_string_lossy().strip_suffix(".yaml").ok_or("unexpected filename")?.to_owned();
    let data = fs::read_to_string(path.as_ref())?;
    let tests = YamlLoader::load_from_str(&data)?;
    let tests = tests[0].as_vec().unwrap();
    let mut error_count = 0;

    let mut test = yaml::Hash::new();
    for (idx, test_data) in tests.iter().enumerate() {
        let desc = format!("{}-{}", test_name, idx);
        let is_xfail = EXPECTED_FAILURES.contains(&desc.as_str());

        // Test fields except `fail` are "inherited"
        let test_data = test_data.as_hash().unwrap();
        test.remove(&Yaml::String("fail".into()));
        for (key, value) in test_data.clone() {
            test.insert(key, value);
        }

        if let Some(error) = run_single_test(&test) {
            if !is_xfail {
                eprintln!("[{}] {}", desc, error);
                error_count += 1;
            }
        } else {
            if is_xfail {
                eprintln!("[{}] UNEXPECTED PASS", desc);
                error_count += 1;
            }
        }
    }
    Ok(error_count)
}

fn run_single_test(test: &yaml::Hash) -> Option<String> {
    if test.get(&Yaml::String("skip".into())).is_some() {
        return None;
    }
    let source = test[&Yaml::String("yaml".into())].as_str().unwrap();
    let should_fail = test.get(&Yaml::String("fail".into())) == Some(&Yaml::Boolean(true));
    let actual_events = parse_to_events(&yaml_to_raw(source));
    if should_fail {
        if actual_events.is_ok() {
            return Some(format!("no error while expected"));
        }
    } else {
        let expected_events = yaml_to_raw(test[&Yaml::String("tree".into())].as_str().unwrap());
        match actual_events {
            Ok(events) => {
                if let Some(diff) = events_differ(events, &expected_events) {
                    //dbg!(source, yaml_to_raw(source));
                    return Some(format!("events differ: {}", diff));
                }
            }
            Err(error) => {
                //dbg!(source, yaml_to_raw(source));
                return Some(format!("unexpected error {:?}", error));
            }
        }
    }
    None
}

fn parse_to_events(source: &str) -> Result<Vec<String>, ScanError> {
    let mut reporter = EventReporter::new();
    Parser::new(source.chars())
        .load(&mut reporter, true)?;
    Ok(reporter.events)
}

struct EventReporter {
    events: Vec<String>,
}

impl EventReporter {
    fn new() -> Self {
        Self {
            events: vec![],
        }
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
                format!("=VAL{}{} {}{}",
                    format_index(idx), format_tag(tag), kind, escape_text(text))
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
                    Some(format!("line {} differs: expected `{}`, found `{}`", idx, exp, act))
                }
            }
            (Some(a), None) => Some(format!("extra actual line: {:?}", a)),
            (None, Some(e)) => Some(format!("extra expected line: {:?}", e)),
            (None, None) => None,
        }
    }
    unreachable!()
}

/// Replace the unprintable characters used in the YAML examples with normal
fn yaml_to_raw(yaml: &str) -> String {
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
    expected_tree.split("\n")
        .map(|s| s.trim_start().to_owned())
        .filter(|s| !s.is_empty())
        .map(|mut s| {
            // Anchor name-to-number conversion
            if let Some(start) = s.find("&") {
                if s[..start].find(":").is_none() {
                    let len = s[start..].find(" ").unwrap_or(s[start..].len());
                    anchors.push(s[start+1..start + len].to_owned());
                    s = s.replace(&s[start..start + len], &format!("&{}", anchors.len()));
                }
            }
            // Alias nodes name-to-number
            if s.starts_with("=ALI") {
                let start = s.find("*").unwrap();
                let name = &s[start + 1 ..];
                let idx = anchors.iter().enumerate().filter(|(_, v)| v == &name).last().unwrap().0;
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

static EXPECTED_FAILURES: &[&str] = &[
    // These seem to be API limited (not enough information on the event stream level)
    // No tag available for SEQ and MAP
    "2XXW-0",
    "35KP-0",
    "57H4-0",
    "6JWB-0",
    "735Y-0",
    "9KAX-0",
    "BU8L-0",
    "C4HZ-0",
    "EHF6-0",
    "J7PZ-0",
    "UGM3-0",
    // Cannot resolve tag namespaces
    "5TYM-0",
    "6CK3-0",
    "6WLZ-0",
    "9WXW-0",
    "CC74-0",
    "U3C3-0",
    "Z9M4-0",
    "P76L-0", // overriding the `!!` namespace!
];
