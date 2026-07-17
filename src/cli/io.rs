use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Open input as a BufReader. Reads from stdin if path is None.
pub fn open_input(path: Option<&Path>) -> io::Result<Box<dyn Read>> {
    match path {
        Some(p) => Ok(Box::new(File::open(p)?)),
        None => Ok(Box::new(io::stdin())),
    }
}

/// Open output as a BufWriter. Writes to stdout if path is None.
pub fn open_output(path: Option<&Path>) -> io::Result<Box<dyn Write>> {
    match path {
        Some(p) => Ok(Box::new(BufWriter::new(File::create(p)?))),
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}

/// Wrap a Read in a BufReader.
pub fn buf_reader(reader: impl Read + 'static) -> BufReader<Box<dyn Read>> {
    BufReader::new(Box::new(reader))
}
