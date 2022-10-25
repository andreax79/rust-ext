use crate::cmds::Options;
use crate::fs::open_filesystem;
use argparse::ArgumentParser;
use humansize::{file_size_opts as options, FileSize};
use std::io::{self, Error};

fn parse_args(args: Vec<String>) {
    // Parse command argument
    let mut parser = ArgumentParser::new();
    parser.set_description("Show information about the file system.");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

pub fn df(options: &Options, args: Vec<String>) -> Result<(), Error> {
    let fs = open_filesystem(&options.filename)?;
    parse_args(args);
    let size = fs.get_blocks_count() * fs.get_blocksize();
    let avail = fs.get_free_blocks_count() * fs.get_blocksize();
    let used = size - avail;
    let use_percent = 100.0 * used as f32 / size as f32;
    println!("Filesystem                        Size           Used      Avail     Use%");
    println!(
        "{:30} {:12} {:12} {:12} {:.0}%",
        options.filename,
        size.file_size(options::DECIMAL).unwrap(),
        used.file_size(options::DECIMAL).unwrap(),
        avail.file_size(options::DECIMAL).unwrap(),
        use_percent,
    );
    Ok(())
}
