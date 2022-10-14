use crate::cmds::Options;
use crate::fs::Ext2Filesystem;
use crate::file::FsFile;
use argparse::{ArgumentParser, List};
use std::io::{self, Error, Read};

fn parse_args(args: Vec<String>, paths: &mut Vec<String>) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Concatenate FILE(s) to standard output.");
    parser.refer(paths).add_argument("file", List, "FILE");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

const BUFFER_SIZE: usize = 1024;

pub fn get_char(ch: u8) -> char {
    if ch >= 32 && ch < 127 {
        char::from_u32(ch as u32).expect("")
    } else {
        '.'
    }
}

pub fn print_file(f: &mut FsFile) -> Result<(), Error> {
    // Print file content on the standard output
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    loop {
        let len = f.read(&mut buffer)?;
        if len == 0 {
            break;
        }
        for b in 0 .. (len+15) / 16 {
            let addr = b * 16;
            let t = &buffer[addr..addr+16];
            print!("{:08x} ", addr);
            for ch in t {
                print!(" {:02x}", ch);
            }
            print!("  |");
            for ch in t {
                print!("{}", get_char(*ch));
            }
            println!("|");
        }
    }
    Ok(())
}

pub fn show_file(path: &String, fs: &Ext2Filesystem) -> Result<(), Error> {
    // Open a file and print the content on the standard output
    match FsFile::open(&fs, path) {
        Ok(mut f) => {
            print_file(&mut f)?;
        },
        Err(x) => {
            eprintln!("hd: {}: {}", path, x);
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn hd(options: &Options, args: Vec<String>) -> Result<(), Error> {
    // Parse command argument
    let mut paths: Vec<String> = vec![];
    parse_args(args, &mut paths);
    let fs = Ext2Filesystem::open(&options.filename)?;
    for path in paths.iter() {
        show_file(path, &fs)?;
    }
    Ok(())
}
