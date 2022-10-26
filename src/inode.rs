use crate::dir::DirEntry;
use crate::disk::Disk;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::io::Error;

pub trait Inode {
    /// Read a directory
    fn read_dir(
        &self,
        disk: &Box<dyn Disk>,
        path: &str,
    ) -> Result<BTreeMap<String, Box<dyn DirEntry>>, Error>;
    /// Block numbers
    fn get_blocks(&self, disk: &Box<dyn Disk>) -> Result<Vec<u64>, Error>;
    /// Block size in bytes
    fn get_block_size(&self) -> u64;
    /// Size in bytes
    fn get_size(&self) -> u64;
    /// Given a path, query the file system to get information about a file, directory, etc.
    fn metadata(&self) -> Metadata;
}
