#![allow(dead_code)]

mod gen;
mod nested;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use rand::{rngs::SmallRng, Rng, SeedableRng};

/// The path into which the generated YAML files will be written.
const OUTPUT_DIR: &str = "bench_yaml";

fn main() -> std::io::Result<()> {
    let mut generator = Generator::new();
    let output_path = Path::new(OUTPUT_DIR);
    if !output_path.is_dir() {
        std::fs::create_dir(output_path).unwrap();
    }

    println!("Generating big.yaml");
    let mut out = BufWriter::new(File::create(output_path.join("big.yaml")).unwrap());
    generator.gen_record_array(&mut out, 100_000, 100_001)?;

    println!("Generating nested.yaml");
    let mut out = BufWriter::new(File::create(output_path.join("nested.yaml")).unwrap());
    nested::create_deep_object(&mut out, 1_100_000)?;

    println!("Generating small_objects.yaml");
    let mut out = BufWriter::new(File::create(output_path.join("small_objects.yaml")).unwrap());
    generator.gen_authors_array(&mut out, 4_000_000, 4_000_001)?;

    println!("Generating strings_array.yaml");
    let mut out = BufWriter::new(File::create(output_path.join("strings_array.yaml")).unwrap());
    generator.gen_strings_array(&mut out, 1_300_000, 1_300_001, 10, 40)?;
    Ok(())
}

/// YAML Generator.
struct Generator {
    /// The RNG state.
    ///
    /// We don't need to be cryptographically secure. [`SmallRng`] also implements the
    /// [`SeedableRng`] trait, allowing runs to be predictible.
    rng: SmallRng,
    /// The stack of indentations.
    indents: Vec<usize>,
}

type GenFn<W> = dyn FnOnce(&mut Generator, &mut W) -> std::io::Result<()>;

impl Generator {
    /// Create a new generator.
    fn new() -> Self {
        Generator {
            rng: SmallRng::seed_from_u64(42),
            indents: vec![0],
        }
    }

    /// Generate an array of records as per [`Self::gen_record_object`].
    fn gen_record_array<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        items_lo: usize,
        items_hi: usize,
    ) -> std::io::Result<()> {
        self.gen_array(writer, items_lo, items_hi, Generator::gen_record_object)
    }

    /// Generate an array of lipsum one-liners.
    fn gen_strings_array<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        items_lo: usize,
        items_hi: usize,
        words_lo: usize,
        words_hi: usize,
    ) -> std::io::Result<()> {
        self.gen_array(writer, items_lo, items_hi, |gen, writer| {
            write!(writer, "{}", gen::words(&mut gen.rng, words_lo, words_hi))
        })
    }

    /// Generate a YAML object/mapping containing a record.
    ///
    /// Fields are description, hash, version, home, repository and pdf.
    /// The `description` field is a long string and puts a lot of weight in plain scalar / block
    /// scalar parsing.
    fn gen_record_object<W: std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        let fields: Vec<(String, Box<GenFn<W>>)> = vec![
            (
                "description".to_string(),
                Box::new(|gen, w| {
                    write!(w, "|")?;
                    gen.push_indent(2);
                    gen.nl(w)?;
                    let indent = gen.indent();
                    let text = gen::text(&mut gen.rng, 1, 9, 3, 8, 10, 20, 80 - indent);
                    gen.write_lines(w, &text)?;
                    gen.pop_indent();
                    Ok(())
                }),
            ),
            (
                "authors".to_string(),
                Box::new(|gen, w| {
                    gen.push_indent(2);
                    gen.nl(w)?;
                    gen.gen_authors_array(w, 1, 10)?;
                    gen.pop_indent();
                    Ok(())
                }),
            ),
            (
                "hash".to_string(),
                Box::new(|gen, w| write!(w, "{}", gen::hex_string(&mut gen.rng, 64))),
            ),
            (
                "version".to_string(),
                Box::new(|gen, w| write!(w, "{}", gen::integer(&mut gen.rng, 1, 9))),
            ),
            (
                "home".to_string(),
                Box::new(|gen, w| {
                    write!(w, "{}", gen::url(&mut gen.rng, "https", 0, 1, 0, 0, None))
                }),
            ),
            (
                "repository".to_string(),
                Box::new(|gen, w| {
                    write!(w, "{}", gen::url(&mut gen.rng, "git", 1, 4, 10, 20, None))
                }),
            ),
            (
                "pdf".to_string(),
                Box::new(|gen, w| {
                    write!(
                        w,
                        "{}",
                        gen::url(&mut gen.rng, "https", 1, 4, 10, 30, Some("pdf"))
                    )
                }),
            ),
        ];
        self.gen_object(writer, fields)
    }

    /// Generate an array of authors as per [`Self::gen_author_object`].
    fn gen_authors_array<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        items_lo: usize,
        items_hi: usize,
    ) -> std::io::Result<()> {
        self.gen_array(writer, items_lo, items_hi, Generator::gen_author_object)
    }

    /// Generate a small object with 2 string fields.
    fn gen_author_object<W: std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        let fields: Vec<(String, Box<GenFn<W>>)> = vec![
            (
                "name".to_string(),
                Box::new(|gen, w| write!(w, "{}", gen::full_name(&mut gen.rng, 10, 15))),
            ),
            (
                "email".to_string(),
                Box::new(|gen, w| write!(w, "{}", gen::email(&mut gen.rng, 1, 9))),
            ),
        ];
        self.gen_object(writer, fields)
    }

    /// Generate a YAML array/sequence containing nodes generated by the given function.
    fn gen_array<W: std::io::Write, F: FnMut(&mut Generator, &mut W) -> std::io::Result<()>>(
        &mut self,
        writer: &mut W,
        len_lo: usize,
        len_hi: usize,
        mut obj_creator: F,
    ) -> std::io::Result<()> {
        let mut first = true;
        for _ in 0..self.rng.gen_range(len_lo..len_hi) {
            if first {
                first = false;
            } else {
                self.nl(writer)?;
            }
            write!(writer, "- ")?;
            self.push_indent(2);
            (obj_creator)(self, writer)?;
            self.pop_indent();
        }
        Ok(())
    }

    /// Create a Yaml object with some fields in it.
    fn gen_object<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        fields: Vec<(String, Box<GenFn<W>>)>,
    ) -> std::io::Result<()> {
        let mut first = true;
        for (key, f) in fields {
            if first {
                first = false;
            } else {
                self.nl(writer)?;
            }
            write!(writer, "{key}: ")?;
            f(self, writer)?;
        }
        Ok(())
    }

    /// Write the given lines at the right indentation.
    fn write_lines<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        lines: &[String],
    ) -> std::io::Result<()> {
        let mut first = true;

        for line in lines {
            if first {
                first = false;
            } else {
                self.nl(writer)?;
            }
            write!(writer, "{line}")?;
        }

        Ok(())
    }

    /// Write a new line to the writer and indent.
    fn nl<W: std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer)?;
        for _ in 0..self.indent() {
            write!(writer, " ")?;
        }
        Ok(())
    }

    /// Return the given indent.
    fn indent(&self) -> usize {
        *self.indents.last().unwrap()
    }

    /// Push a new indent with the given relative offset.
    fn push_indent(&mut self, offset: usize) {
        self.indents.push(self.indent() + offset);
    }

    /// Pops the last indent.
    fn pop_indent(&mut self) {
        self.indents.pop();
        assert!(!self.indents.is_empty());
    }
}
