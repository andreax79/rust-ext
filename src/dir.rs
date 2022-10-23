pub trait DirEntry {
    // Returns the bare file name of this directory entry without any other leading path component
    fn file_name(&self) -> String;
    // Returns the inode number
    fn inode_num(&self) -> u32;
}

#[derive(Debug)]
pub struct DefaultDirEntry {
    pub file_name: String, // file name
    pub inode_num: u32,    // inode number
}

impl DirEntry for DefaultDirEntry {
    fn file_name(&self) -> String {
        // Returns the bare file name of this directory entry without any other leading path component
        return self.file_name.clone();
    }

    fn inode_num(&self) -> u32 {
        // Returns the inode number
        return self.inode_num;
    }
}
