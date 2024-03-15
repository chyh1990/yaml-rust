use std::{fs::File, io::BufWriter, io::Write, path::Path};

use anyhow::Error;
use serde::{Deserialize, Serialize};

fn main() {
    if let Err(e) = entrypoint() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}

fn entrypoint() -> Result<(), Error> {
    let config: Config =
        toml::from_str(&std::fs::read_to_string("bench_compare.toml").unwrap()).unwrap();
    if config.parsers.is_empty() {
        println!("Please add at least one parser. Refer to the README for instructions.");
        return Ok(());
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2
        || (args.len() == 2 && !["time_parse", "run_bench"].contains(&args[1].as_str()))
    {
        println!("Usage: bench_compare <time_parse|run_bench>");
        return Ok(());
    }
    match args[1].as_str() {
        "run_bench" => run_bench(&config)?,
        "time_parse" => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

/// Run the `run_bench` binary on the given parsers.
fn run_bench(config: &Config) -> Result<(), Error> {
    // Create output directory
    std::fs::create_dir_all(&config.yaml_output_dir)?;

    let inputs = list_input_files(config)?;
    let iterations = format!("{}", config.iterations);
    let mut averages = vec![];

    // Inputs are ordered, so are parsers.
    for input in &inputs {
        let input_basename = Path::new(&input).file_name().unwrap().to_string_lossy();
        let mut input_times = vec![];

        // Run each input for each parser.
        for parser in &config.parsers {
            println!("Running {input_basename} against {}", parser.name);
            // Run benchmark
            let path = Path::new(&parser.path).join("run_bench");
            let output = std::process::Command::new(path)
                .arg(input)
                .arg(&iterations)
                .arg("--output-yaml")
                .output()?;
            // Check exit status.
            if output.status.code().unwrap_or(1) == 0 {
                let s = String::from_utf8_lossy(&output.stdout);
                // Get output as yaml.
                match serde_yaml::from_str::<BenchYamlOutput>(&s) {
                    Ok(output) => {
                        // Push average into our CSV-to-be.
                        input_times.push(output.average);
                        // Save the YAML for later.
                        serde_yaml::to_writer(
                            BufWriter::new(File::create(format!(
                                "{}/{}-{}",
                                config.yaml_output_dir, parser.name, input_basename
                            ))?),
                            &output,
                        )?;
                    }
                    Err(e) => {
                        // Yaml is invalid, use 0 as "didn't run properly".
                        println!("Errored: Invalid YAML output: {e}");
                        input_times.push(0);
                    }
                }
            } else {
                // An error happened, use 0 as "didn't run properly".
                println!("Errored: process did exit non-zero");
                input_times.push(0);
            }
        }
        averages.push(input_times);
    }

    // Finally, save a CSV.
    save_run_bench_csv(config, &inputs, &averages)
}

/// General configuration structure.
#[derive(Serialize, Deserialize)]
struct Config {
    /// The path to the directory containing the input yaml files.
    yaml_input_dir: String,
    /// Number of iterations to run, if using `run_bench`.
    iterations: u32,
    /// The parsers to run.
    parsers: Vec<Parser>,
    /// The path to the directory in which `run_bench`'s yamls are saved.
    yaml_output_dir: String,
    /// The path to the CSV output aggregating times for each parser and file.
    csv_output: String,
}

/// A parser configuration.
#[derive(Serialize, Deserialize)]
struct Parser {
    /// The name of the parser.
    name: String,
    /// The path in which the parser's `run_bench` and `time_parse` are located.
    path: String,
}

/// Ourput of running `run_bench` on a given parser.
#[derive(Serialize, Deserialize)]
struct BenchYamlOutput {
    /// The name of the parser.
    parser: String,
    /// The file taken as input.
    input: String,
    /// Average parsing time (ns).
    average: u64,
    /// Shortest parsing time (ns).
    min: u64,
    /// Longest parsing time (ns).
    max: u64,
    /// 95th percentile of parsing times (ns).
    percentile95: u64,
    /// Number of iterations.
    iterations: u64,
    /// Parsing times for each run.
    times: Vec<u64>,
}

/// Save a CSV file with all averages from `run_bench`.
fn save_run_bench_csv(
    config: &Config,
    inputs: &[String],
    averages: &[Vec<u64>],
) -> Result<(), Error> {
    let mut csv = BufWriter::new(File::create(&config.csv_output)?);
    for parser in &config.parsers {
        write!(csv, ",{}", parser.name,)?;
    }
    writeln!(csv)?;
    for (path, averages) in inputs.iter().zip(averages.iter()) {
        let filename = Path::new(path).file_name().unwrap().to_string_lossy();
        write!(csv, "{}", filename)?;
        for avg in averages {
            write!(csv, ",{avg}")?;
        }
        writeln!(csv)?;
    }

    Ok(())
}

/// Returns the paths to the input yaml files.
fn list_input_files(config: &Config) -> Result<Vec<String>, Error> {
    Ok(std::fs::read_dir(&config.yaml_input_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_string_lossy().to_string())
        .filter(|path| {
            Path::new(path)
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("yaml"))
        })
        .collect())
}
