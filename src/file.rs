use crate::fs::Ext2Filesystem;
use crate::disk::Offset;
use crate::inode::Inode;
use std::io::{ /*BufRead,*/ Error, ErrorKind, Read } ;

// #[derive(Debug)]
pub struct FsFile<'a> {
    fs: &'a Ext2Filesystem,
    inode: Inode,
    blocks: Vec<u32>,
    pos: u64,
}

impl FsFile<'_> {
    pub fn open<'a>(fs: &'a Ext2Filesystem, path: &str) -> Result<FsFile<'a>, Error> {
        let inode = fs.resolve(path)?;
        if inode.is_dir() {
            Err(Error::new(ErrorKind::InvalidInput, "Is a directory"))
        } else {
            let blocks = inode.get_blocks(&fs.disk)?;
            Ok(FsFile{ fs: fs, inode: inode, blocks: blocks, pos: 0 })
        }
    }

    fn read_file_block(&mut self, file_block_num: u32) -> Result<Vec<u8>, Error> {
        let offset = Offset::Block {
            block_size: self.inode.block_size,
            block_num: self.blocks[file_block_num as usize],
        };
        self.fs.disk.read(self.inode.block_size, offset)
    }
}

impl Read for FsFile<'_> {

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Pull some bytes from this file into the specified buffer,
        // returning how many bytes were read
        if self.pos >= self.inode.ext2_inode.i_size as u64 {
            // End of file
            let zero: Vec<u8> = vec![0; buf.len()];
            buf.copy_from_slice(&zero[..]);
            Ok(0)
        } else {
            let len = if self.pos + buf.len() as u64 > self.inode.ext2_inode.i_size as u64 {
                (self.inode.ext2_inode.i_size as u64 - self.pos) as usize
            } else {
                buf.len()
            };
            let block_num: u32 = (self.pos / self.inode.block_size as u64) as u32;
            let mut buffer = self.read_file_block(block_num)?;
            if len < buf.len() {
                // If len < buffer length, fill the buffer with 0
                let zero: Vec<u8> = vec![0; buf.len() - len];
                buffer.extend_from_slice(&zero);
            }
            let pos: usize = (self.pos - block_num as u64 * self.inode.block_size as u64) as usize;
            // println!("block num: {} len: {} pos: {}", block_num, len, pos);
            buf.copy_from_slice(&buffer[pos..pos+buf.len()]);
            self.pos += len as u64;
            Ok(len)
        }
    }
}

// impl BufRead for FsFile<'_> {
//     fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
//         Err(Error::new(ErrorKind::NotFound, "TODO"))
//     }
//
//     fn consume(&mut self, amt: usize) {
//     }
// }
