use std::fs;
use std::path;
// use std::fs::File;
// use std::io::{Write, BufReader, BufRead};
use std::io::Write;
use clap::{ArgGroup, Parser};


#[derive(Parser)]
/// Filtering one file based on a fuzzy column in another
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("outs")
    .required(true)
    .args(["inplace", "output"]),
))]
struct Args {
    /// File to filter with queries
    #[arg(short, long, value_parser = file_exists)]
    target: String,

    /// Patterns to remove from the target file
    #[arg(short, long, value_parser = file_exists)]
    query: String,

    /// Filtering starts after skipping <skip_n> lines
    #[arg(short = 'n', long, default_value_t = 0)]
    skip_n: usize,

    /// The column starts after the specified char (or string)
    #[arg(short = 'c', long, default_value_t = String::from(""))]
    after_char: String,

    /// If true, target file will be overwritten in place
    #[arg(long = "in-place", default_value_t = false)]
    inplace: bool,

    /// Name of the output file
    #[arg(short, long, value_parser = file_exists)]
    output: Option<String>,

    /// By default, the first line of the query file is skipped
    #[arg(long = "no-header", default_value_t = false)]
    no_header: bool,
}


// Don't start bulky computations unless the file checked exists.
fn file_exists(name: &str) -> Result<String,String> {
    let mut err_msg = "This path does not exist.".to_string();
    if path::Path::new(name).exists() {
        if fs::metadata(name).unwrap().is_file() {
            return Ok(name.to_string());
        } else {
            err_msg = format!("Expected a file, {name} is a folder.");
        }
    }
    Err(err_msg)
}


// Extension for various formats of the `query` file.
// Currently, we use the first pattern below with `sep=" "`.
// My first macro in Rust, by the way.
macro_rules! stringfix {  // like string's (pre/suf)fix
    // Enclosing with extra braces allows using multiple statements.
    ( $string: expr, $sep: expr $(, $(1,)? slice = "suffix")? ) => {{
        let anchor = $string.find($sep).unwrap_or(0) + 1;
        $string[anchor.min($string.len())..].to_string()
    }};

    // Not sure about this use case (if it is optimal and necessary).
    ( $string: expr, $sep: expr, slice = "prefix" ) => {{
        let anchor = $string.find($sep).unwrap_or($string.len());
        $string[..anchor].to_string()
    }};

    // No check like in the pattern below that `col` is within the string.
    ( $string: expr, $sep: expr, $col: expr $(, slice = "unit")? ) => {
        $string.split($sep).collect::<Vec<_>>()[$col].to_string()
    };

    // Allow taking prefixes bigger than the input string.
    ( $string: expr, $sep: expr, $col: expr, slice = "prefix" ) => {{
        let words = $string.split($sep).collect::<Vec<_>>();
        words[..($col + 1).min(words.len())].join($sep).to_string()
    }};

    // No check like in the pattern above that `col` is within the string.
    ( $string: expr, $sep: expr, $col: expr, slice = "suffix" ) => {
        $string.split($sep).collect::<Vec<_>>()[$col..]
            .join($sep).to_string()
    };
}


fn main () -> std::io::Result<()> {
    let args = Args::parse();
    let mut output_name = args.target.clone();
    if !args.inplace { output_name = args.output.unwrap(); }

    // Read target and query files substituting non-UTF-8 characters.
    let target = fs::read(args.target)?;
    let target = String::from_utf8_lossy(&target);
    let target: Vec<&str> = target.split("\n")
        .map(|l| l.trim()).collect();
    // Just in case. Trimming is on Zsh side ('histreduceblanks' option).

    let mut buf = Vec::<String>::with_capacity(target.capacity());

    let query = fs::read(args.query)?;
    let query = String::from_utf8_lossy(&query);
    let query: Vec<String> = query.split("\n")
        .skip(!args.no_header as usize).filter(|l| l.len() > 0)
        .map(|l| stringfix!(l, " ").trim().to_string()).collect();
        //        ↑ Extract a command string from `pat`. ↑
        // Note: the zeroth column is the exit status of the command.

    // ↓ This can't work with non-UTF-8 characters. ↓
    // let target: Vec<String> = BufReader::new(
    //     File::open(&args.target)?).lines()
    //     .map(|l| l.unwrap()).collect();

    // let mut buf = Vec::<String>::with_capacity(target.capacity());

    // let query: Vec<String> = BufReader::new(
    //     File::open(args.query)?).lines()
    //     .map(|l| l.unwrap()).collect();

    for (i, line) in target.iter().enumerate() {
        // Clumsy 'if' - will be removed in the future.
        if i < args.skip_n {
            buf.push(line.to_string());
            continue;
        }

        let mut add_flag = true;
        for pat in query.iter() {
            if line.eq(pat) {
                add_flag = false;
                break;
            }
        }

        if add_flag {
            buf.push(line.to_string());
        }
    }

    let buf = buf.join("\n");
    write!(fs::File::create(output_name)?, "{buf}")?;
    Ok(())
}


// Tests for `stringfix` macro.
#[cfg(test)]
mod tests {
    const INPUT: &str = "this is a test string";

    #[test]
    fn default_for_suffix() {
        let answer = "is a test string";
        assert_eq!(stringfix!(INPUT, " "), answer);
        assert_eq!(stringfix!(INPUT, " ", slice="suffix"), answer);
        assert_eq!(stringfix!(INPUT, " ", 1, slice="suffix"), answer);
    }

    #[test]
    fn default_for_prefix() {
        assert_eq!(stringfix!(INPUT, " ", slice="prefix"), "this");
    }

    #[test]
    fn default_for_unit() {
        let answer = "a";
        assert_eq!(stringfix!(INPUT, " ", 2), answer);
        assert_eq!(stringfix!(INPUT, " ", 2, slice="unit"), answer);
    }

    #[test]
    fn not_defaults() {
        let answer_1 = "this is a test";
        let answer_2 = "test string";
        assert_eq!(stringfix!(INPUT, " ", 3, slice="prefix"), answer_1);
        assert_eq!(stringfix!(INPUT, " ", 3, slice="suffix"), answer_2);
    }

    #[test]
    fn trailing_sep() {
        let input = "anything ";
        assert_eq!(stringfix!(input, " "), "");
        assert_eq!(stringfix!(input, " ", slice="prefix"), input.trim_end());
    }
}
