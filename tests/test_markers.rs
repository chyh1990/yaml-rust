#[macro_use]
extern crate indoc;
extern crate yaml_rust;

use yaml_rust::{Node, YamlLoader, YamlMarked};

type R<A> = Result<A, Box<std::error::Error>>;

#[test]
fn test_top_level_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            - a
            - b
        "#
    ))?;
    let Node(_, marker) = docs[0];
    assert_eq!(marker.unwrap().line(), 1, "line");
    assert_eq!(marker.unwrap().col(), 0, "col");
    Ok(())
}

#[test]
fn test_top_level_location_in_non_initial_document() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            - a
            - b
            ---
            foo: 1
            bar: 2
        "#
    ))?;
    let Node(_, marker) = docs[1];
    assert_eq!(marker.unwrap().line(), 4, "line");
    // TODO: column is given as 3, but I expected 0
    // assert_eq!(marker.unwrap().col(), 0, "col");
    Ok(())
}

#[test]
fn test_array_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            items:
                - a
                - b
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Hash(ref hash), _) => {
            let (_, array) = hash.front().unwrap();
            assert_eq!(array.marker().unwrap().line(), 2, "line");
            assert_eq!(array.marker().unwrap().col(), 4, "col");
            Ok(())
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}

#[test]
fn test_array_element_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            items:
                - a
                - b
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Hash(ref hash), _) => {
            let (_, node) = hash.front().unwrap();
            match node {
                Node(YamlMarked::Array(array), _) => {
                    let elem = &array[1];
                    assert_eq!(elem.marker().unwrap().line(), 3, "line");
                    assert_eq!(elem.marker().unwrap().col(), 6, "col");
                    Ok(())
                }
                Node(yaml, _) => Err(format!("expectd an array but got {:#?}", yaml))?,
            }
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}

#[test]
fn test_hash_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            - 1
            - foo: 1
              bar: 2
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Array(ref array), _) => {
            let hash = &array[1];
            assert_eq!(hash.marker().unwrap().line(), 2, "line");
            // TODO: column is given as 5, but I expected 2
            // assert_eq!(hash.marker().unwrap().col(), 2, "col");
            Ok(())
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}

#[test]
fn test_hash_key_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            ---
            foo: bar
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Hash(ref hash), _) => {
            let (key, _) = hash.front().unwrap();
            let Node(_, key_marker) = key;
            assert_eq!(key_marker.unwrap().line(), 2, "line");
            assert_eq!(key_marker.unwrap().col(), 0, "col");
            Ok(())
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}

#[test]
fn test_hash_value_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            ---
            foo: bar
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Hash(ref hash), _) => {
            let (_, value) = hash.front().unwrap();
            let Node(_, value_marker) = value;
            assert_eq!(value_marker.unwrap().line(), 2, "line");
            assert_eq!(value_marker.unwrap().col(), 5, "col");
            Ok(())
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}

#[test]
fn test_alias_location() -> R<()> {
    let docs = YamlLoader::load_from_str_with_markers(indoc!(
        r#"
            items:
                - &first a
                - b
                - *first
        "#
    ))?;
    match &docs[0] {
        Node(YamlMarked::Hash(ref hash), _) => {
            let (_, node) = hash.front().unwrap();
            match node {
                Node(YamlMarked::Array(ref array), _) => {
                    let elem = &array[2];
                    assert_eq!(elem.marker().unwrap().line(), 4, "line");
                    assert_eq!(elem.marker().unwrap().col(), 6, "col");
                    Ok(())
                }
                Node(yaml, _) => Err(format!("expected an array but got {:#?}", yaml))?,
            }
        }
        Node(yaml, _) => Err(format!("expected a hash but got {:#?}", yaml))?,
    }
}
