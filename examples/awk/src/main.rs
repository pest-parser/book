use anyhow::Result;
use clap::{Arg, Command};
use std::fs;
use std::io::{self, Read};

// Import our modules
mod ast;
mod interpreter;
mod parser;

use interpreter::*;
use parser::*;

fn main() -> Result<()> {
    let matches = Command::new("awk")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A simple AWK clone built with pest")
        .long_about("
This AWK clone demonstrates how to build a complete language interpreter using Rust and pest.
It supports AWK's core features including pattern-action programming, field processing,
built-in variables, and control flow constructs.

Examples:
  awk-clone -p '{ print $1, $2 }' data.txt
  awk-clone -f script.awk input.txt
  cat data.txt | awk-clone -p 'BEGIN { sum = 0 } { sum += $2 } END { print sum }'
        ")
        .arg(Arg::new("program")
            .short('p')
            .long("program")
            .value_name("PROGRAM")
            .help("AWK program string")
            .long_help("AWK program as a command-line string. Cannot be used with --file.")
            .required_unless_present("file")
            .conflicts_with("file"))
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("AWK program file")
            .long_help("Read AWK program from a file. Cannot be used with --program.")
            .required_unless_present("program")
            .conflicts_with("program"))
        .arg(Arg::new("input")
            .help("Input file (reads from stdin if not provided)")
            .long_help("Input data file to process. If not provided, reads from standard input.")
            .index(1))
        .arg(Arg::new("field-separator")
            .short('F')
            .long("field-separator")
            .value_name("FS")
            .help("Field separator pattern")
            .long_help("Set the field separator. Default is whitespace. Examples: -F ',' for CSV, -F ':' for /etc/passwd"))
        .get_matches();

    // Parse program source
    let program_text = if let Some(prog) = matches.get_one::<String>("program") {
        prog.clone()
    } else if let Some(file) = matches.get_one::<String>("file") {
        fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("Failed to read program file '{}': {}", file, e))?
    } else {
        unreachable!("clap should ensure either program or file is provided");
    };

    // Read input data
    let input_text = if let Some(input_file) = matches.get_one::<String>("input") {
        fs::read_to_string(input_file)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", input_file, e))?
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
        buffer
    };

    // Parse the AWK program
    let program =
        parse_program(&program_text).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Create and configure interpreter
    let mut interpreter = Interpreter::new();

    if let Some(fs) = matches.get_one::<String>("field-separator") {
        interpreter.set_field_separator(fs.clone());
    }

    // Execute the program
    interpreter
        .run_program(&program, &input_text)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

    Ok(())
}
