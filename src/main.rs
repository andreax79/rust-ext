pub mod dir;
pub mod disk;
pub mod group;
pub mod inode;
pub mod superblock;

use crate::dir::DirEntry;
use crate::disk::Disk;
use crate::group::Ext2BlockGroups;
use crate::inode::Inode;
use crate::superblock::Ext2SuperBlock;
use chrono::prelude::*;
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::process;
use std::str;

const FILENAME: &str = "root";
const EXT2_ROOT_INO: u32 = 2; /* Root inode */

pub struct FS {
    pub disk: Disk,
    pub super_block: Ext2SuperBlock,
    pub block_groups: Ext2BlockGroups,
}

impl FS {
    fn open(filename: &str) -> FS {
        let mut disk = Disk::open(filename);
        let super_block = Ext2SuperBlock::new(&mut disk);
        let block_groups = Ext2BlockGroups::new(&mut disk, &super_block);
        FS {
            disk: disk,
            super_block: super_block,
            block_groups: block_groups,
        }
    }
    fn read_inode(&mut self, inode_num: u32) -> Inode {
        Inode::new(
            &mut self.disk,
            self.super_block.s_inode_size as usize,
            self.super_block.get_blocksize(),
            &self.block_groups,
            inode_num,
        )
    }
    fn resolve(&mut self, path: &str) -> Option<Inode> {
        let mut inode = self.read_inode(EXT2_ROOT_INO);
        for part in path.split("/") {
            if !part.is_empty() {
                match inode.get_child(&mut self.disk, &self.block_groups, part) {
                    Some(child) => inode = child,
                    None => return None,
                }
            }
        }
        Some(inode)
    }
    fn readdir(&mut self, path: &str) -> Result<BTreeMap<String, DirEntry>, &'static str> {
        match self.resolve(path) {
            Some(inode) => inode.readdir(&mut self.disk),
            None => Err("No such file or directory"),
        }
    }
    fn cat(&mut self, path: &str) {
        match self.resolve(path) {
            Some(inode) => {
                for block in inode.read_blocks(&mut self.disk) {
                    io::stdout()
                        .write(&block)
                        .expect("Unable to write on stdout");
                }
            }
            None => {
                eprintln!("No such file or directory");
                process::exit(1);
            }
        }
    }
    fn ls(&mut self, path: &str) {
        match self.readdir(path) {
            Ok(entries) => {
                println!("  Inode    Mode Link   Uid   Gid     Side Last modification   File name");
                println!(
                    "---------------------------------------------------------------------------"
                );
                for (_, entry) in entries.iter() {
                    let inode = self.read_inode(entry.inode_num);
                    println!(
                        "{:7} {:7o} {:4} {:5} {:5} {:8} {:19} {}",
                        inode.inode_num,
                        inode.ext2_inode.i_mode,
                        inode.ext2_inode.i_links_count,
                        inode.ext2_inode.i_uid,
                        inode.ext2_inode.i_gid,
                        inode.ext2_inode.i_size,
                        format_time(inode.ext2_inode.i_mtime),
                        entry.file_name,
                    )
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    }
}

fn format_time(time: u32) -> String {
    let naive = NaiveDateTime::from_timestamp(time.into(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut fs = FS::open(FILENAME);
    if args.len() > 1 {
        let path = &args[1];
        match fs.resolve(path) {
            Some(inode) => {
                if inode.is_dir() {
                    fs.ls(path);
                } else if inode.is_file() {
                    fs.cat(path);
                }
            }
            None => {
                eprintln!("No such file or directory");
                process::exit(1);
            }
        }
    }
}
