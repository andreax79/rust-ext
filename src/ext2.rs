pub mod dir;
pub mod group;
pub mod inode;
pub mod superblock;

use crate::dir::DirEntry;
use crate::disk::{Disk, FileDisk};
use crate::ext2::group::Ext2BlockGroups;
use crate::ext2::inode::Ext2Inode;
use crate::ext2::superblock::Ext2SuperBlock;
use crate::file::FsFile;
use crate::fs::Filesystem;
use crate::inode::Inode;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str;

const EXT2_ROOT_INO: u64 = 2; /* Root inode */

pub struct Ext2Filesystem {
    disk: Box<dyn Disk>,
    super_block: Ext2SuperBlock,
    block_groups: Ext2BlockGroups,
}

impl Ext2Filesystem {
    pub fn mount(filename: &str) -> Result<Ext2Filesystem, Error> {
        let disk = FileDisk::open(filename)?;
        let super_block = Ext2SuperBlock::new(&disk)?;
        let block_groups = Ext2BlockGroups::new(&disk, &super_block)?;
        Ok(Ext2Filesystem {
            disk: Box::new(disk),
            super_block: super_block,
            block_groups: block_groups,
        })
    }

    /// Get inode by number
    fn read_inode(&self, inode_num: u64) -> Result<Ext2Inode, Error> {
        Ext2Inode::new(
            &self.disk,
            self.super_block.s_inode_size as u64,
            self.super_block.get_block_size(),
            &self.block_groups,
            inode_num,
        )
    }

    /// Get inode by path
    fn resolve(&self, path: &str) -> Result<Ext2Inode, Error> {
        let root_inode = self.read_inode(EXT2_ROOT_INO)?;
        self.resolve_relative(path, root_inode, false)
    }

    /// Get inode by relative path
    fn resolve_relative(
        &self,
        path: &str,
        mut inode: Ext2Inode,
        link: bool,
    ) -> Result<Ext2Inode, Error> {
        if path.starts_with("/") {
            // if the path is absolute, resolve from root inode
            inode = self.read_inode(EXT2_ROOT_INO)?;
        }
        let path_parts: Vec<_> = path.split("/").collect();
        let last = path_parts.len() - 1;
        for (i, part) in path_parts.iter().enumerate() {
            if !part.is_empty() {
                match inode.get_child(&self.disk, &self.block_groups, part) {
                    Some(child) => {
                        let resolve_symlink = child.metadata().is_symlink() && (!link || i != last);
                        if resolve_symlink {
                            let target = child.read_link(&self.disk)?;
                            inode = self.resolve_relative(&target, inode, link)?;
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
}

impl Filesystem for Ext2Filesystem {
    fn open(&self, path: &str) -> Result<FsFile, Error> {
        let inode = self.resolve(path)?;
        if inode.metadata().is_dir() {
            Err(Error::new(ErrorKind::InvalidInput, "Is a directory"))
        } else {
            let blocks = inode.get_blocks(&self.disk)?;
            Ok(FsFile::new(&self.disk, Box::new(inode), blocks))
        }
    }

    /// Get block size
    fn get_block_size(&self) -> u64 {
        self.super_block.get_block_size()
    }

    /// Get the number of blocks in file system
    fn get_blocks_count(&self) -> u64 {
        self.super_block.s_blocks_count as u64
    }

    /// Get the number of unallocated blocks
    fn get_free_blocks_count(&self) -> u64 {
        self.super_block.s_free_blocks_count as u64
    }

    /// Read the contents of a given directory
    fn read_dir(&self, path: &str) -> Result<BTreeMap<String, Box<dyn DirEntry>>, Error> {
        let inode = self.resolve(path)?;
        inode.read_dir(&self.disk, path)
    }

    /// Given a path, query the file system to get information about a file, directory, etc.
    fn metadata(&self, path: &str) -> Result<Metadata, Error> {
        let root_inode = self.read_inode(EXT2_ROOT_INO)?;
        let inode = self.resolve_relative(path, root_inode, true)?;
        Ok(inode.metadata())
    }

    /// Like stat, except that if path is a symbolic link, then the link itself is stat-ed,
    /// not the file that it refers to.
    fn symlink_metadata(&self, path: &str) -> Result<Metadata, Error> {
        let root_inode = self.read_inode(EXT2_ROOT_INO)?;
        let inode = self.resolve_relative(path, root_inode, true)?;
        Ok(inode.metadata())
    }

    /// Reads a symbolic link, returning the file that the link points to
    fn read_link(&self, path: &str) -> Result<String, Error> {
        // Read value of a symbolic link
        let root_inode = self.read_inode(EXT2_ROOT_INO)?;
        let inode = self.resolve_relative(path, root_inode, true)?;
        inode.read_link(&self.disk)
    }
}
