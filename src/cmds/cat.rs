use crate::fs::Ext2Filesystem;
use argparse::{ArgumentParser, List};
use std::io::{self, Error, Write};
use std::str;

fn parse_args(args: Vec<String>, paths: &mut Vec<String>) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Concatenate FILE(s) to standard output.");
    parser.refer(paths).add_argument("file", List, "FILE");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

pub fn cat(filename: &str, args: Vec<String>) -> Result<(), Error> {
    let mut fs = Ext2Filesystem::open(filename)?;
    let mut paths: Vec<String> = vec![];
    parse_args(args, &mut paths);
    for path in paths.iter() {
        match fs.resolve(path) {
            Ok(inode) => {
                if inode.is_dir() {
                    eprintln!("cat: {}: Is a directory", path);
                    std::process::exit(1);
                }
                let mut t: Vec<u8> = Vec::new();
                for block in inode.read_blocks(&mut fs.disk) {
                    let block = block?;
                    t.extend(&block);
                    io::stdout()
                        .write(&block)
                        .expect("Unable to write on stdout");
                }
                io::stdout().write(&t).expect("Unable to write on stdout");
            }
            Err(x) => {
                eprintln!("cat: {}: {}", path, x);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
