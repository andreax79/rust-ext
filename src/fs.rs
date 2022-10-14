use crate::dir::DirEntry;
use crate::disk::Disk;
use crate::group::Ext2BlockGroups;
use crate::inode::Inode;
use crate::superblock::Ext2SuperBlock;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::ErrorKind;
use std::str;

const EXT2_ROOT_INO: u32 = 2; /* Root inode */

pub struct Ext2Filesystem {
    pub disk: Disk,
    pub super_block: Ext2SuperBlock,
    block_groups: Ext2BlockGroups,
}

impl Ext2Filesystem {
    pub fn open(filename: &str) -> Result<Ext2Filesystem, Error> {
        let disk = Disk::open(filename)?;
        let super_block = Ext2SuperBlock::new(&disk)?;
        let block_groups = Ext2BlockGroups::new(&disk, &super_block)?;
        Ok(Ext2Filesystem {
            disk: disk,
            super_block: super_block,
            block_groups: block_groups,
        })
    }
    pub fn read_inode(&self, inode_num: u32) -> Result<Inode, Error> {
        Inode::new(
            &self.disk,
            self.super_block.s_inode_size as usize,
            self.super_block.get_blocksize(),
            &self.block_groups,
            inode_num,
        )
    }
    pub fn resolve(&self, path: &str) -> Result<Inode, Error> {
        let inode = self.read_inode(EXT2_ROOT_INO)?;
        self.resolve_relative(path, inode)
    }
    pub fn resolve_relative(&self, path: &str, mut inode: Inode) -> Result<Inode, Error> {
        if path.starts_with("/") {
            // if the path is absolute, resolve from root inode
            inode = self.read_inode(EXT2_ROOT_INO)?;
        }
        for part in path.split("/") {
            if !part.is_empty() {
                match inode.get_child(&self.disk, &self.block_groups, part) {
                    Some(child) => {
                        if child.is_symlink() {
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
    pub fn readdir(&self, path: &str) -> Result<BTreeMap<String, DirEntry>, Error> {
        let inode = self.resolve(path)?;
        inode.readdir(&self.disk)
    }
}
