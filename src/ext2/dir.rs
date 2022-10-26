use crate::dir::DirEntry;
use std::io::Read;
use std::mem;
use std::slice;
use std::str;

#[repr(C)]
#[derive(Debug, Default)]
struct Ext2DirEntryStruct {
    inode_num: u32, // Inode number
    rec_len: u16,   // Directory entry length
    name_len: u8,   // Name length
    file_type: u8,  // Type indicator
                    // (only if the feature bit for "directory entries have file type byte" is set)
}

// Directory entry
#[derive(Debug)]
pub struct Ext2DirEntry {
    path: String,      // full path
    file_name: String, // file name
    inode_num: u64,    // inode number
}
impl Ext2DirEntry {
    pub fn new(buffer: &Vec<u8>, offset: usize, parent: &str) -> (Ext2DirEntry, usize) {
        let mut ext2_dir_entry = Ext2DirEntryStruct::default();
        let size = mem::size_of::<Ext2DirEntryStruct>();
        let mut buf = &buffer[offset..offset + size];
        let p = &mut ext2_dir_entry as *mut _ as *mut u8;
        unsafe {
            let dir_slice = slice::from_raw_parts_mut(p, size);
            buf.read_exact(dir_slice).unwrap();
        }
        let name_slice = &buffer[offset + size..offset + size + ext2_dir_entry.name_len as usize];
        let name = match str::from_utf8(name_slice) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        let dir_entry = Ext2DirEntry {
            path: if parent != "/" {
                format!("{}/{}", parent, name)
            } else {
                format!("/{}", name)
            },
            file_name: String::from(name),
            inode_num: ext2_dir_entry.inode_num as u64,
        };
        (dir_entry, ext2_dir_entry.rec_len as usize)
    }
}

impl DirEntry for Ext2DirEntry {
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
