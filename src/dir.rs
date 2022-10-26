pub trait DirEntry {
    // Returns the full path to the file that this entry represents.
    fn path(&self) -> String;
    // Returns the bare file name of this directory entry without any other leading path component
    fn file_name(&self) -> String;
    // Returns the inode number
    fn inode_num(&self) -> u64;
}

#[derive(Debug)]
pub struct DefaultDirEntry {
    pub path: String,      // full path
    pub file_name: String, // file name
    pub inode_num: u64,    // inode number
}

impl DirEntry for DefaultDirEntry {
    fn path(&self) -> String {
        // Returns the full path to the file that this entry represents.
        return self.path.clone();
    }

    fn file_name(&self) -> String {
        // Returns the bare file name of this directory entry without any other leading path component
        return self.file_name.clone();
    }

    fn inode_num(&self) -> u64 {
        // Returns the inode number
        return self.inode_num;
    }
}
