pub mod cmds;
pub mod dir;
pub mod disk;
pub mod file;
pub mod fs;
pub mod group;
pub mod inode;
pub mod superblock;

use crate::cmds::{Command, Options};
use crate::disk::Disk;
use argparse::{ArgumentParser, List, Store};
use std::env;
use std::io;
use std::str;

const FILENAME: &str = "root";

fn get_cmd() -> String {
    let args: Vec<String> = env::args().collect();
    args[0].clone()
}

fn parse_args(subcommand: &mut Command, args: &mut Vec<String>, options: &mut Options) {
    // Parse command argument
    let mut parser = ArgumentParser::new();
    parser
        .refer(&mut options.filename)
        .required()
        .add_argument("device", Store, "Device");
    parser
        .refer(subcommand)
        .required()
        .add_argument("command", Store, "Command");
    parser
        .refer(args)
        .add_argument("arguments", List, "Arguments for command");
    parser.stop_on_first_argument(true);
    if let Err(x) = parser.parse(env::args().collect(), &mut io::stdout(), &mut io::sink()) {
        eprintln!("Usage:");
        eprintln!("  {} DEVICE COMMAND [ARGUMENTS ...]", get_cmd());
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  cat              Concatenate FILE(s) to standard output.");
        eprintln!("  df               Show information about the file system.");
        eprintln!("  hd               Display file contents in hexadecimal.");
        eprintln!("  ls               List information about the FILEs.");
        std::process::exit(x);
    }
}

fn main() {
    let mut options: Options = Options {
        filename: String::from(FILENAME),
    };
    let mut subcommand = Command::ls;
    let mut args = vec![];
    parse_args(&mut subcommand, &mut args, &mut options);
    args.insert(0, format!("{} {:?}", get_cmd(), subcommand));
    let result = subcommand.run_command(&options, args);
    match result {
        Ok(_) => std::process::exit(0),
        Err(x) => {
            eprintln!("{}: {}", get_cmd(), x);
            std::process::exit(1);
        }
    }
}
