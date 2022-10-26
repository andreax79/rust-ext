use crate::dir::DirEntry;
use crate::ext2::Ext2Filesystem;
use crate::file::FsFile;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::io::{Error, Read};

pub trait Filesystem {
    /// Open a file
    fn open(&self, path: &str) -> Result<FsFile, Error>;
    /// Get block size
    fn get_block_size(&self) -> u64;
    /// Get the number of blocks in file system
    fn get_blocks_count(&self) -> u64;
    /// Get the number of unallocated blocks
    fn get_free_blocks_count(&self) -> u64;
    /// Read the contents of a given directory
    fn read_dir(&self, path: &str) -> Result<BTreeMap<String, Box<dyn DirEntry>>, Error>;
    /// Given a path, query the file system to get information about a file, directory, etc.
    fn metadata(&self, path: &str) -> Result<Metadata, Error>;
    /// Like stat, except that if path is a symbolic link, then the link itself is stat-ed,
    /// not the file that it refers to.
    fn symlink_metadata(&self, path: &str) -> Result<Metadata, Error>;
    /// Read value of a symbolic link
    fn read_link(&self, path: &str) -> Result<String, Error>;
}

impl dyn Filesystem {
    /// Read the entire contents of a file into a bytes vector
    pub fn read(&self, path: &str) -> Result<Vec<u8>, Error> {
        let mut file = self.open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Read the entire contents of a file into a string
    pub fn read_to_string(&self, path: &str) -> Result<String, Error> {
        let mut file = self.open(path)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(string)
    }

    /// Returns Ok(true) if the path points at an existing entity
    pub fn try_exists(&self, path: &str) -> Result<bool, Error> {
        self.open(path)?;
        Ok(true)
    }
}

pub fn mount(filename: &str) -> Result<Box<dyn Filesystem>, Error> {
    Ok(Box::new(Ext2Filesystem::mount(filename)?))
}
