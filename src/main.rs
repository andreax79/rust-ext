pub mod cmds;
pub mod dir;
pub mod disk;
pub mod fs;
pub mod group;
pub mod inode;
pub mod superblock;

use std::str;
use std::env;
use argparse::{ArgumentParser, List, Store};
use crate::disk::Disk;

const FILENAME: &str = "root";

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = &args[0].clone();
    let mut filename = String::from(FILENAME);
    let mut subcommand = String::new(); // = Command::ls;
    let mut args = vec![];
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut filename)
            .add_option(&["--device"], Store, "Device");
        parser
            .refer(&mut subcommand)
            .required()
            .add_argument("command", Store, "cat, df, ls");
        parser
            .refer(&mut args)
            .add_argument("arguments", List, r#"Arguments for command"#);
        parser.stop_on_first_argument(true);
        parser.parse_args_or_exit();
    }

    let filename = filename.as_str();
    args.insert(0, format!("{} {:?}", cmd, subcommand));
    let result = match subcommand.as_str() {
        "cat" => cmds::cat::cat(filename, args),
        "df" => cmds::df::df(filename, args),
        "ls" => cmds::ls::ls(filename, args),
        &_ => todo!(),
    };
    match result {
        Ok(_) => std::process::exit(0),
        Err(x) => {
            eprintln!("{}: {}", cmd, x);
            std::process::exit(1);
        }
    }
}
