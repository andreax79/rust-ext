use crate::cmds::Options;
use crate::file::FsFile;
use crate::fs::{mount, Filesystem};
use argparse::{ArgumentParser, List};
use std::io::{self, Error, Read, Write};

fn parse_args(args: Vec<String>, paths: &mut Vec<String>) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Concatenate FILE(s) to standard output.");
    parser.refer(paths).add_argument("file", List, "FILE");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

const BUFFER_SIZE: usize = 1024;

pub fn print_file(f: &mut FsFile) -> Result<(), Error> {
    // Print file content on the standard output
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    loop {
        let len = f.read(&mut buffer)?;
        if len == 0 {
            break;
        }
        io::stdout()
            .write(&buffer)
            .expect("Unable to write on stdout");
    }
    Ok(())
}

pub fn cat_file(path: &String, fs: &Box<dyn Filesystem>) -> Result<(), Error> {
    // Open a file and print the content on the standard output
    match fs.open(path) {
        Ok(mut f) => {
            print_file(&mut f)?;
        }
        Err(x) => {
            eprintln!("cat: {}: {}", path, x);
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn cat(options: &Options, args: Vec<String>) -> Result<(), Error> {
    // Parse command argument
    let mut paths: Vec<String> = vec![];
    parse_args(args, &mut paths);
    let fs = mount(&options.filename)?;
    for path in paths.iter() {
        cat_file(path, &fs)?;
    }
    Ok(())
}
