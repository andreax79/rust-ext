pub mod dir;
pub mod group;
pub mod superblock;

use crate::dir::DirEntry;
use crate::disk::{Disk, FileDisk};
use crate::ext2::group::Ext2BlockGroups;
use crate::ext2::superblock::Ext2SuperBlock;
use crate::fs::Filesystem;
use crate::inode::Inode;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str;

const EXT2_ROOT_INO: u32 = 2; /* Root inode */

pub struct Ext2Filesystem {
    disk: Box<dyn Disk>,
    super_block: Ext2SuperBlock,
    block_groups: Ext2BlockGroups,
}

impl Ext2Filesystem {
    pub fn open(filename: &str) -> Result<Ext2Filesystem, Error> {
        let disk = FileDisk::open(filename)?;
        let super_block = Ext2SuperBlock::new(&disk)?;
        let block_groups = Ext2BlockGroups::new(&disk, &super_block)?;
        Ok(Ext2Filesystem {
            disk: Box::new(disk),
            super_block: super_block,
            block_groups: block_groups,
        })
    }
}

impl Filesystem for Ext2Filesystem {
    fn get_disk(&self) -> &Box<dyn Disk> {
        // Get disk
        return &self.disk;
    }

    fn get_blocksize(&self) -> u32 {
        // Get block size
        self.super_block.get_blocksize()
    }

    fn get_blocks_count(&self) -> u32 {
        // Get the number of blocks in file system
        self.super_block.s_blocks_count
    }

    fn get_free_blocks_count(&self) -> u32 {
        // Get the number of unallocated blocks
        self.super_block.s_free_blocks_count
    }

    fn read_inode(&self, inode_num: u32) -> Result<Inode, Error> {
        // Get inode by number
        Inode::new(
            self.get_disk(),
            self.super_block.s_inode_size as u32,
            self.super_block.get_blocksize(),
            &self.block_groups,
            inode_num,
        )
    }

    fn resolve(&self, path: &str) -> Result<Inode, Error> {
        // Get inode by path
        let inode = self.read_inode(EXT2_ROOT_INO)?;
        self.resolve_relative(path, inode)
    }

    fn resolve_relative(&self, path: &str, mut inode: Inode) -> Result<Inode, Error> {
        // Get inode by relative path
        if path.starts_with("/") {
            // if the path is absolute, resolve from root inode
            inode = self.read_inode(EXT2_ROOT_INO)?;
        }
        for part in path.split("/") {
            if !part.is_empty() {
                match inode.get_child(&self.disk, &self.block_groups, part) {
                    Some(child) => {
                        if child.metadata().is_symlink() {
                            let target = child.readlink(&self.disk)?;
                            inode = self.resolve_relative(&target, inode)?;
                        } else {
                            inode = child
                        }
                    }
                    None => {
                        return Err(Error::new(ErrorKind::NotFound, "No such file or directory"))
                    }
                }
            }
        }
        Ok(inode)
    }

    fn readdir(&self, path: &str) -> Result<BTreeMap<String, Box<dyn DirEntry>>, Error> {
        // Read the contents of a given directory
        let inode = self.resolve(path)?;
        inode.readdir(&self.disk)
    }

    fn metadata(&self, path: &str) -> Result<Metadata, Error> {
        // Given a path, query the file system to get information about a file, directory, etc.
        let inode = self.resolve(path)?;
        Ok(inode.metadata())
    }
}
