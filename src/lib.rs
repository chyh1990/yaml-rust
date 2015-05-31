pub mod yaml;
pub mod scanner;
pub mod parser;
pub mod emitter;

// reexport key APIs
pub use scanner::ScanError;
pub use parser::Event;
pub use yaml::{Yaml, YamlLoader};
pub use emitter::{YamlEmitter, EmitError};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_api() {
        let s =
"
# from yaml-cpp example
- name: Ogre
  position: [0, 5, 0]
  powers:
    - name: Club
      damage: 10
    - name: Fist
      damage: 8
- name: Dragon
  position: [1, 0, 10]
  powers:
    - name: Fire Breath
      damage: 25
    - name: Claws
      damage: 15
- name: Wizard
  position: [5, -3, 0]
  powers:
    - name: Acid Rain
      damage: 50
    - name: Staff
      damage: 3
";
        let docs = YamlLoader::load_from_str(s).unwrap();
        let doc = &docs[0];

        assert_eq!(doc[0]["name"].as_str().unwrap(), "Ogre");

        let mut writer = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut writer);
            emitter.dump(doc).unwrap();
        }

        assert!(writer.len() > 0);
    }

    #[test]
    fn test_fail() {
        let s =
"
# syntax error
scalar
key: [1, 2]]
key1:a2
";
        assert!(YamlLoader::load_from_str(s).is_err());
    }

}
