use crate::fs::Ext2Filesystem;
use argparse::{ArgumentParser, List, Store, StoreTrue};
use chrono::prelude::*;
use std::io::{self, Error};
use std::str;

fn format_time(time: u32) -> String {
    let naive = NaiveDateTime::from_timestamp(time.into(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn parse_args(args: Vec<String>, paths: &mut Vec<String>) {
    let mut parser = ArgumentParser::new();
    parser.set_description("List information about the FILEs.");
    parser.refer(paths).add_argument("file", List, "FILE");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

pub fn ls(filename: &str, args: Vec<String>) -> Result<(), Error> {
    let mut fs = Ext2Filesystem::open(filename)?;
    let mut paths: Vec<String> = vec![];
    parse_args(args, &mut paths);
    if paths.is_empty() {
        paths = vec![String::from("/")];
    }
    for path in paths.iter() {
        match fs.readdir(path) {
            Ok(entries) => {
                println!(
                    "  Inode    Mode    Link   Uid   Gid     Size Last modification   File name"
                );
                println!(
                    "---------------------------------------------------------------------------"
                );
                for (_, entry) in entries.iter() {
                    let inode = fs.read_inode(entry.inode_num)?;
                    let mut tmp = String::new();
                    if inode.is_symlink() {
                        tmp.push_str("-> ");
                        tmp.push_str(&inode.readlink(&mut fs.disk)?);
                    }
                    println!(
                        "{:7} {:10} {:4} {:5} {:5} {:8} {:19} {} {}",
                        inode.inode_num,
                        unix_mode::to_string(inode.ext2_inode.i_mode as u32),
                        inode.ext2_inode.i_links_count,
                        inode.ext2_inode.i_uid,
                        inode.ext2_inode.i_gid,
                        inode.ext2_inode.i_size,
                        format_time(inode.ext2_inode.i_mtime),
                        entry.file_name,
                        tmp,
                    )
                }
            }
            Err(err) => {
                eprintln!("ls: {}: {}", path, err);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
