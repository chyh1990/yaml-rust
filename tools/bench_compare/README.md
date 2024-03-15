# `bench_compare`
This tool helps with comparing times different YAML parsers take to parse the same input.

## Synopsis
```
bench_compare time_parse
bench_compare run_bench
```

This will run either `time_parse` or `run_bench` (described below) with the given set of parsers from the configuration file.

## Parsers requirements
Parsers are expected to be event-based. In order to be fair to this crate's benchmark implementation, parsers should:

* Load the file into memory (a string, `mmap`, ...) **prior** to starting the clock
* Initialize the parser, if needed
* **Start the clock**
* Read events from the parser while the parser has not finished parsing
* Discard events as they are received (dropping them, `free`ing them or anything similar) so as to not grow their memory consumption too high, and allowing the parser to reuse event structures
* **Stop the clock**
* Destroy the resources, if needed/wanted (parser, file buffer, ...). The kernel will reap after the process exits.


## Parsers required binaries
This tool recognizes 2 binaries: `time_parse` and `run_bench`.

### `time_parse`
Synopsis:
```
time_parse file.yaml [--short]
```

The binary must run the aforementioned steps and display on its output the time the parser took to parse the given file.
With the `--short` option, the binary must only output the benchmark time in nanoseconds.

```sh
# This is meant to be human-readable.
# The example below is what this crate implements.
$> time_parse file.yaml
Loaded 200MiB in 1.74389s.

# This will be read by this tool.
# This must output ONLY the time, in nanoseconds.
$> time_parse file.yaml --short
1743892394
```

This tool will always provide the `--short` option.

### `run_bench`
Synopsis:
```
run_bench file.yaml <iterations> [--output-yaml]
```

The binary is expected to run `<iteration>` runs of the aforementioned steps and display on its output relevant information.
The `--output-yaml` instructs the binary to output details about its runs in YAML on its standard output.
The binary may optionally perform some warmup runs prior to running the benchmark. The time it took the binary to run will not be evaluated.

```sh
# This is meant to be human-readable.
# The example below is what this crate implements.
$> run_bench file.yaml 100
Average: 1.589485s
Min    : 1.583078s
Max    : 1.597028s
95%    : 1.593219s

# This will be read by this tool.
# This must output a YAML as described below.
$> run_bench ../file.yaml 10 --output-yaml
parser: yaml-rust2
input: ../file.yaml
average: 1620303590
min: 1611632108
max: 1636401896
percentile95: 1636401896
iterations: 10
times:
  - 1636401896
  - 1623914538
  - 1611632108
  - 1612973608
  - 1617748930
  - 1615419514
  - 1612172250
  - 1620791346
  - 1629339306
  - 1622642412
```

The expected fields are (all times in nanoseconds):

* `parser`: The name of the parser (in case of a mistake renaming files)
* `input`: The path to the input file as given to the binary arguments
* `average`: The average time it took to run the parser
* `min`: The shortest time it took to run the parser
* `max`: The longest time it took to run the parser
* `percentile95`: The 95th percentile time of the runs
* `iterations`: The number of times the parser was run (`<iterations>`)
* `times`: An array of `iterations` times, one for each run, in the order they were run (first run first)

## Configuration
`bench_compare` is configured through a `bench_compare.toml` file. This file must be located in the current directory.
As of now, default values are unsupported and all fields must be set. The following fields are required:
```toml
yaml_input_dir = "bench_yaml" # The path to the directory containing the input yaml files
iterations = 10               # The number of iterations, if using `run_bench`
yaml_output_dir = "yaml_output" # The directory in which `run_bench`'s yamls are saved
csv_output = "benchmark.csv"  # The CSV output aggregating times for each parser and file

[[parsers]]                   # A parser, can be repeated as many times as there are parsers
name = "yaml-rust2"           # The name of the parser (used for logging)
path = "target/release/"      # The path in which the parsers' `run_bench` and `time_parse` are

# If there is another parser, another block can be added
# [[parsers]]
# name = "libfyaml"
# path = "../libfyaml/build"
```
