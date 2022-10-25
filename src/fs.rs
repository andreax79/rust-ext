use crate::dir::DirEntry;
use crate::disk::Disk;
use crate::file::FsFile;
use crate::ext2::Ext2Filesystem;
use crate::inode::Inode;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::io::Error;

pub trait Filesystem {
    // Get disk
    fn get_disk(&self) -> &Box<dyn Disk>;
    // Get block size
    fn get_blocksize(&self) -> u32;
    // Get the number of blocks in file system
    fn get_blocks_count(&self) -> u32;
    // Get the number of unallocated blocks
    fn get_free_blocks_count(&self) -> u32;
    // Get inode by number
    fn read_inode(&self, inode_num: u32) -> Result<Inode, Error>;
    // Get inode by path
    fn resolve(&self, path: &str) -> Result<Inode, Error>;
    // Get inode by relative path
    fn resolve_relative(&self, path: &str, inode: Inode) -> Result<Inode, Error>;
    // Read the contents of a given directory
    fn readdir(&self, path: &str) -> Result<BTreeMap<String, Box<dyn DirEntry>>, Error>;
    // Given a path, query the file system to get information about a file, directory, etc.
    fn metadata(&self, path: &str) -> Result<Metadata, Error>;
}

impl dyn Filesystem {
    pub fn open<'a>(&'a self, path: &str) -> Result<FsFile<'a>, Error> {
        FsFile::open(self, path)
    }
}

pub fn open_filesystem(filename: &str) -> Result<Box<dyn Filesystem>, Error> {
    Ok(Box::new(Ext2Filesystem::open(filename)?))
}
