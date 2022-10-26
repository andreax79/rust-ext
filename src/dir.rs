pub trait DirEntry {
    /// Returns the full path to the file that this entry represents.
    fn path(&self) -> String;
    /// Returns the bare file name of this directory entry without any other leading path component
    fn file_name(&self) -> String;
    /// Returns the inode number
    fn inode_num(&self) -> u64;
}

#[derive(Debug)]
pub struct DefaultDirEntry {
    pub path: String,      // full path
    pub file_name: String, // file name
    pub inode_num: u64,    // inode number
}

impl DirEntry for DefaultDirEntry {
    /// Returns the full path to the file that this entry represents.
    fn path(&self) -> String {
        return self.path.clone();
    }

    /// Returns the bare file name of this directory entry without any other leading path component
    fn file_name(&self) -> String {
        return self.file_name.clone();
    }

    /// Returns the inode number
    fn inode_num(&self) -> u64 {
        return self.inode_num;
    }
}
