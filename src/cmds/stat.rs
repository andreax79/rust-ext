use crate::cmds::Options;
use crate::fs::{open_filesystem, Filesystem};
use argparse::{ArgumentParser, List};
use chrono::prelude::*;
use std::io::{self, Error};
use std::str;

const FMT_LONG: &str = "%Y-%m-%d %H:%M:%S";

fn format_time(time: i64) -> String {
    // Format timestamp
    let naive = NaiveDateTime::from_timestamp(time.into(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format(FMT_LONG).to_string()
}

struct StatFlags {}

fn parse_args(args: Vec<String>, paths: &mut Vec<String>) {
    // Parse command argument
    let mut parser = ArgumentParser::new();
    parser.set_description("Display file status.");
    parser.refer(paths).add_argument("file", List, "FILE");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

fn print_stat(fs: &mut Box<dyn Filesystem>, path: &str, _flags: &StatFlags) -> Result<(), Error> {
    let metadata = fs.lstat(path)?;
    println!("  File: {}", path);
    println!(
        "  Size: {:<14}  Blocks: {:<9}  IO Block: {:<8} {}",
        metadata.size, metadata.blocks, metadata.blksize, metadata.file_type().to_string()
    );
    println!(
        "Device: {0:04x}h/{0:<06}d   Inode: {1:<10}  Links: {2}",
        metadata.dev, metadata.ino, metadata.nlink
    );
    println!(
        "Access: ({0:04o}/{1})  Uid: ({2})   Gid: ({3})",
        metadata.mode,
        unix_mode::to_string(metadata.mode),
        metadata.uid,
        metadata.gid
    );
    println!("Access: {}", format_time(metadata.atime));
    println!("Modify: {}", format_time(metadata.mtime));
    println!("Change: {}", format_time(metadata.ctime));

    Ok(())
}

pub fn stat(options: &Options, args: Vec<String>) -> Result<(), Error> {
    let mut fs = open_filesystem(&options.filename)?;
    let mut paths: Vec<String> = vec![];
    parse_args(args, &mut paths);
    if paths.is_empty() {
        eprintln!("stat: missing operand");
        std::process::exit(1);
    }
    let flags = StatFlags {};
    for path in paths.iter() {
        match print_stat(&mut fs, &path, &flags) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("stat: {}: {}", path, err);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
