use crate::cmds::Options;
use crate::dir::{DefaultDirEntry, DirEntry};
use crate::ext2::Ext2Filesystem;
use crate::fs::Filesystem;
use crate::inode::Inode;
use argparse::{ArgumentParser, List, StoreTrue};
use chrono::prelude::*;
use chrono::Duration;
use std::io::{self, Error};
use std::str;

// const FMT_LONG: &str = "%Y-%m-%d %H:%M:%S";
const FMT_NEAR: &str = "%b %e %H:%M";
const FMT_FAR: &str = "%b %e  %Y";

fn format_time(time: i64) -> String {
    // Format timestamp
    let naive = NaiveDateTime::from_timestamp(time.into(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    if Utc::now() - datetime > Duration::days(365 / 2) {
        datetime.format(FMT_FAR).to_string()
    } else {
        datetime.format(FMT_NEAR).to_string()
    }
}

struct LsFlags {
    long_flg: bool,
    inode_flg: bool,
    size_flg: bool,
}

fn parse_args(
    args: Vec<String>,
    paths: &mut Vec<String>,
    long_flg: &mut bool,
    inode_flg: &mut bool,
    size_flg: &mut bool,
) {
    // Parse command argument
    let mut parser = ArgumentParser::new();
    parser.set_description("List information about the FILEs.");
    parser.refer(paths).add_argument("file", List, "FILE");
    parser
        .refer(long_flg)
        .add_option(&["-l", "--long"], StoreTrue, "use a long listing format");
    parser.refer(inode_flg).add_option(
        &["-i", "--inode"],
        StoreTrue,
        "print the index number of each file",
    );
    parser.refer(size_flg).add_option(
        &["-s", "--size"],
        StoreTrue,
        "print the allocated size of each file, in blocks",
    );
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

fn print_direntry(
    fs: &mut dyn Filesystem,
    entry: &Box<dyn DirEntry>,
    flags: &LsFlags,
) -> Result<(), Error> {
    let inode = fs.read_inode(entry.inode_num())?;
    let metadata = inode.metadata();
    let mut prefix = String::new();
    if flags.inode_flg {
        prefix.push_str(&format!("{:7 } ", metadata.ino));
    }
    if flags.size_flg {
        prefix.push_str(&format!("{:7 } ", metadata.blocks));
    }
    let mut suffix = String::new();
    if metadata.is_symlink() {
        suffix = format!("-> {}", inode.readlink(fs.get_disk())?);
    }
    if flags.long_flg {
        println!(
            "{}{:10} {:4} {:5} {:5} {:8} {} {} {}",
            prefix,
            unix_mode::to_string(metadata.mode),
            metadata.nlink,
            metadata.uid,
            metadata.gid,
            metadata.size,
            format_time(metadata.mtime),
            entry.file_name(),
            suffix,
        )
    } else {
        print!("{} {}  ", prefix, entry.file_name());
    }
    Ok(())
}

fn print_dir(fs: &mut dyn Filesystem, inode: &Inode, flags: &LsFlags) -> Result<(), Error> {
    let entries = inode.readdir(fs.get_disk())?;
    for entry in entries.values() {
        print_direntry(fs, &entry, flags)?
    }
    Ok(())
}

fn print_path(fs: &mut dyn Filesystem, path: &str, flags: &LsFlags) -> Result<(), Error> {
    let inode = fs.resolve(path)?;
    if inode.metadata().is_dir() {
        print_dir(fs, &inode, flags)
    } else {
        let entry: Box<dyn DirEntry> = Box::new(DefaultDirEntry {
            file_name: String::from(path),
            inode_num: inode.inode_num,
        });
        print_direntry(fs, &entry, flags)
    }
}

pub fn ls(options: &Options, args: Vec<String>) -> Result<(), Error> {
    let mut fs = Ext2Filesystem::open(&options.filename)?;
    let mut paths: Vec<String> = vec![];
    let mut long_flg = false;
    let mut inode_flg = false;
    let mut size_flg = false;
    parse_args(
        args,
        &mut paths,
        &mut long_flg,
        &mut inode_flg,
        &mut size_flg,
    );
    if paths.is_empty() {
        paths = vec![String::from("/")];
    }
    let flags = LsFlags {
        long_flg,
        inode_flg,
        size_flg,
    };
    for path in paths.iter() {
        match print_path(&mut fs, &path, &flags) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ls: {}: {}", path, err);
                std::process::exit(1);
            }
        }
    }
    if !long_flg {
        println!();
    }
    Ok(())
}
