pub mod cmds;
pub mod dir;
pub mod disk;
pub mod fs;
pub mod group;
pub mod inode;
pub mod superblock;

use crate::cmds::{Command, Options};
use crate::disk::Disk;
use argparse::{ArgumentParser, List, Store};
use std::env;
use std::str;

const FILENAME: &str = "root";

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
        .add_argument("command", Store, "cat, df, ls");
    parser
        .refer(args)
        .add_argument("arguments", List, "Arguments for command");
    parser.stop_on_first_argument(true);
    parser.parse_args_or_exit();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = &args[0].clone();
    let mut options: Options = Options {
        filename: String::from(FILENAME),
    };
    let mut subcommand = Command::ls;
    let mut args = vec![];
    parse_args(&mut subcommand, &mut args, &mut options);
    args.insert(0, format!("{} {:?}", cmd, subcommand));
    let result = subcommand.run_command(&options, args);
    match result {
        Ok(_) => std::process::exit(0),
        Err(x) => {
            eprintln!("{}: {}", cmd, x);
            std::process::exit(1);
        }
    }
}
