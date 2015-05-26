#[test]
fn test_ex2_1_seq_scalars() {
    let mut p = Parser::new(EX2_1.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_2_mapping_scalars_to_scalars() {
    let mut p = Parser::new(EX2_2.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_3_mapping_scalars_to_sequences() {
    let mut p = Parser::new(EX2_3.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_4_sequence_of_mappings() {
    let mut p = Parser::new(EX2_4.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_5_sequence_of_sequences() {
    let mut p = Parser::new(EX2_5.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_6_mapping_of_mappings() {
    let mut p = Parser::new(EX2_6.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_7_two_documents_in_a_stream() {
    let mut p = Parser::new(EX2_7.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_8_play_by_play_feed() {
    let mut p = Parser::new(EX2_8.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_9_single_document_with_two_comments() {
    let mut p = Parser::new(EX2_9.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_10_simple_anchor() {
    let mut p = Parser::new(EX2_10.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_11_mapping_between_sequences() {
    let mut p = Parser::new(EX2_11.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_12_compact_nested_mapping() {
    let mut p = Parser::new(EX2_12.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_13_in_literals_newlines_are_preserved() {
    let mut p = Parser::new(EX2_13.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_14_in_folded_scalars_newlines_become_spaces() {
    let mut p = Parser::new(EX2_14.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines() {
    let mut p = Parser::new(EX2_15.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_16_indentation_determines_scope() {
    let mut p = Parser::new(EX2_16.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_17_quoted_scalars() {
    let mut p = Parser::new(EX2_17.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_18_multi_line_flow_scalars() {
    let mut p = Parser::new(EX2_18.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_23_various_explicit_tags() {
    let mut p = Parser::new(EX2_23.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_24_global_tags() {
    let mut p = Parser::new(EX2_24.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_25_unordered_sets() {
    let mut p = Parser::new(EX2_25.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_26_ordered_mappings() {
    let mut p = Parser::new(EX2_26.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_27_invoice() {
    let mut p = Parser::new(EX2_27.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex2_28_log_file() {
    let mut p = Parser::new(EX2_28.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_3_block_structure_indicators() {
    let mut p = Parser::new(EX5_3.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_4_flow_structure_indicators() {
    let mut p = Parser::new(EX5_4.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_6_node_property_indicators() {
    let mut p = Parser::new(EX5_6.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_7_block_scalar_indicators() {
    let mut p = Parser::new(EX5_7.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_8_quoted_scalar_indicators() {
    let mut p = Parser::new(EX5_8.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_11_line_break_characters() {
    let mut p = Parser::new(EX5_11.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_12_tabs_and_spaces() {
    let mut p = Parser::new(EX5_12.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex5_13_escaped_characters() {
    let mut p = Parser::new(EX5_13.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_1_indentation_spaces() {
    let mut p = Parser::new(EX6_1.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_2_indentation_indicators() {
    let mut p = Parser::new(EX6_2.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_3_separation_spaces() {
    let mut p = Parser::new(EX6_3.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_4_line_prefixes() {
    let mut p = Parser::new(EX6_4.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_5_empty_lines() {
    let mut p = Parser::new(EX6_5.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_6_line_folding() {
    let mut p = Parser::new(EX6_6.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_7_block_folding() {
    let mut p = Parser::new(EX6_7.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_8_flow_folding() {
    let mut p = Parser::new(EX6_8.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_9_separated_comment() {
    let mut p = Parser::new(EX6_9.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_12_separation_spaces_ii() {
    let mut p = Parser::new(EX6_12.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_13_reserved_directives() {
    let mut p = Parser::new(EX6_13.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_14_yaml_directive() {
    let mut p = Parser::new(EX6_14.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_16_tag_directive() {
    let mut p = Parser::new(EX6_16.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_18_primary_tag_handle() {
    let mut p = Parser::new(EX6_18.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_19_secondary_tag_handle() {
    let mut p = Parser::new(EX6_19.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_20_tag_handles() {
    let mut p = Parser::new(EX6_20.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_21_local_tag_prefix() {
    let mut p = Parser::new(EX6_21.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_22_global_tag_prefix() {
    let mut p = Parser::new(EX6_22.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_23_node_properties() {
    let mut p = Parser::new(EX6_23.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_24_verbatim_tags() {
    let mut p = Parser::new(EX6_24.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_26_tag_shorthands() {
    let mut p = Parser::new(EX6_26.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_28_non_specific_tags() {
    let mut p = Parser::new(EX6_28.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex6_29_node_anchors() {
    let mut p = Parser::new(EX6_29.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_1_alias_nodes() {
    let mut p = Parser::new(EX7_1.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_2_empty_nodes() {
    let mut p = Parser::new(EX7_2.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_3_completely_empty_nodes() {
    let mut p = Parser::new(EX7_3.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_4_double_quoted_implicit_keys() {
    let mut p = Parser::new(EX7_4.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_5_double_quoted_line_breaks() {
    let mut p = Parser::new(EX7_5.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_6_double_quoted_lines() {
    let mut p = Parser::new(EX7_6.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_7_single_quoted_characters() {
    let mut p = Parser::new(EX7_7.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_8_single_quoted_implicit_keys() {
    let mut p = Parser::new(EX7_8.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_9_single_quoted_lines() {
    let mut p = Parser::new(EX7_9.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_10_plain_characters() {
    let mut p = Parser::new(EX7_10.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_11_plain_implicit_keys() {
    let mut p = Parser::new(EX7_11.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_12_plain_lines() {
    let mut p = Parser::new(EX7_12.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_13_flow_sequence() {
    let mut p = Parser::new(EX7_13.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_14_flow_sequence_entries() {
    let mut p = Parser::new(EX7_14.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_15_flow_mappings() {
    let mut p = Parser::new(EX7_15.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_16_flow_mapping_entries() {
    let mut p = Parser::new(EX7_16.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_17_flow_mapping_separate_values() {
    let mut p = Parser::new(EX7_17.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_18_flow_mapping_adjacent_values() {
    let mut p = Parser::new(EX7_18.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_19_single_pair_flow_mappings() {
    let mut p = Parser::new(EX7_19.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_20_single_pair_explicit_entry() {
    let mut p = Parser::new(EX7_20.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_21_single_pair_implicit_entries() {
    let mut p = Parser::new(EX7_21.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_23_flow_content() {
    let mut p = Parser::new(EX7_23.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex7_24_flow_nodes() {
    let mut p = Parser::new(EX7_24.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnAlias);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_1_block_scalar_header() {
    let mut p = Parser::new(EX8_1.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_2_block_indentation_header() {
    let mut p = Parser::new(EX8_2.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_4_chomping_final_line_break() {
    let mut p = Parser::new(EX8_4.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_6_empty_scalar_chomping() {
    let mut p = Parser::new(EX8_6.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_7_literal_scalar() {
    let mut p = Parser::new(EX8_7.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_8_literal_content() {
    let mut p = Parser::new(EX8_8.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_9_folded_scalar() {
    let mut p = Parser::new(EX8_9.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_10_folded_lines() {
    let mut p = Parser::new(EX8_10.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_11_more_indented_lines() {
    let mut p = Parser::new(EX8_11.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_12_empty_separation_lines() {
    let mut p = Parser::new(EX8_12.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_13_final_empty_lines() {
    let mut p = Parser::new(EX8_13.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_14_block_sequence() {
    let mut p = Parser::new(EX8_14.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_15_block_sequence_entry_types() {
    let mut p = Parser::new(EX8_15.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_16_block_mappings() {
    let mut p = Parser::new(EX8_16.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_17_explicit_block_mapping_entries() {
    let mut p = Parser::new(EX8_17.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_18_implicit_block_mapping_entries() {
    let mut p = Parser::new(EX8_18.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnNull);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_19_compact_block_mappings() {
    let mut p = Parser::new(EX8_19.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_20_block_node_types() {
    let mut p = Parser::new(EX8_20.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

#[test]
fn test_ex8_22_block_collection_nodes() {
    let mut p = Parser::new(EX8_22.chars());
    let mut v = yaml_to_test_events(&p.load().unwrap()).into_iter();
    assert_next!(v, TestEvent::OnDocumentStart);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnSequenceEnd);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapStart);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnScalar);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnMapEnd);
    assert_next!(v, TestEvent::OnDocumentEnd);
}

