use std::os::unix::fs::MetadataExt;

#[derive(Debug, Default)]
pub struct Metadata {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: u64,
    pub atime: i64,
    pub atime_nsec: i64,
    pub mtime: i64,
    pub mtime_nsec: i64,
    pub ctime: i64,
    pub ctime_nsec: i64,
    pub blksize: u64,
    pub blocks: u64,
}

impl Metadata {
    pub fn file_type(&self) -> FileType {
        // Returns the file type for this metadata.
        FileType(self.mode)
    }

    pub fn is_dir(&self) -> bool {
        // Tests whether this inode is a directory
        self.file_type().is_dir()
    }

    pub fn is_file(&self) -> bool {
        // Tests whether this inode is a regular file
        self.file_type().is_file()
    }

    pub fn is_symlink(&self) -> bool {
        // Tests whether this inode is a symbolic link
        self.file_type().is_symlink()
    }

    pub fn len(&self) -> u64 {
        // Returns the size of the file, in bytes
        self.size
    }
}

impl MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        self.dev
    }
    fn ino(&self) -> u64 {
        self.ino
    }
    fn mode(&self) -> u32 {
        self.mode
    }
    fn nlink(&self) -> u64 {
        self.nlink
    }
    fn uid(&self) -> u32 {
        self.uid
    }
    fn gid(&self) -> u32 {
        self.gid
    }
    fn rdev(&self) -> u64 {
        self.rdev
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn atime(&self) -> i64 {
        self.atime
    }
    fn atime_nsec(&self) -> i64 {
        self.atime_nsec
    }
    fn mtime(&self) -> i64 {
        self.mtime
    }
    fn mtime_nsec(&self) -> i64 {
        self.mtime_nsec
    }
    fn ctime(&self) -> i64 {
        self.ctime
    }
    fn ctime_nsec(&self) -> i64 {
        self.ctime_nsec
    }
    fn blksize(&self) -> u64 {
        self.blksize
    }
    fn blocks(&self) -> u64 {
        self.blocks
    }
}

pub struct FileType(u32);

impl FileType {
    // Tests whether this inode is a directory
    pub fn is_dir(&self) -> bool {
        return unix_mode::is_dir(self.0);
    }

    // Tests whether this inode is a regular file
    pub fn is_file(&self) -> bool {
        return unix_mode::is_file(self.0);
    }

    // Tests whether this inode is a symbolic link
    pub fn is_symlink(&self) -> bool {
        return unix_mode::is_symlink(self.0);
    }

    // Returns true if this mode represents a fifo, also known as a named pipe.
    pub fn is_fifo(&self) -> bool {
        return unix_mode::is_fifo(self.0);
    }

    // Returns true if this mode represents a character device.
    pub fn is_char_device(&self) -> bool {
        return unix_mode::is_char_device(self.0);
    }

    // Returns true if this mode represents a block device.
    pub fn is_block_device(&self) -> bool {
        return unix_mode::is_block_device(self.0);
    }

    // Returns true if this mode represents a Unix-domain socket.
    pub fn is_socket(&self) -> bool {
        return unix_mode::is_socket(self.0);
    }
}
