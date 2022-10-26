use crate::disk::{Disk, Offset};
use crate::fs::Filesystem;
use crate::inode::Inode;
use std::io::{/*BufRead,*/ Error, Read};

pub struct FsFile<'a> {
    disk: &'a Box<dyn Disk>,
    inode: Box<dyn Inode>,
    blocks: Vec<u64>,
    pos: u64,
}

impl FsFile<'_> {
    pub fn new<'a>(disk: &'a Box<dyn Disk>, inode: Box<dyn Inode>, blocks: Vec<u64>) -> FsFile<'a> {
        FsFile {
            disk: disk,
            inode: inode,
            blocks: blocks,
            pos: 0,
        }
    }

    /// Attempts to open a file
    pub fn open<'a>(fs: &'a dyn Filesystem, path: &str) -> Result<FsFile<'a>, Error> {
        fs.open(path)
    }

    fn read_file_block(&mut self, file_block_num: u64) -> Result<Vec<u8>, Error> {
        let offset = Offset::Block {
            block_size: self.inode.get_block_size(),
            block_num: self.blocks[file_block_num as usize],
        };
        self.disk.read(self.inode.get_block_size(), offset)
    }

    fn how_many_bytes(&self, buffer_len: usize) -> usize {
        if self.pos + buffer_len as u64 > self.inode.get_size() {
            (self.inode.get_size() - self.pos) as usize
        } else {
            buffer_len
        }
    }

    /// If len < buffer length, fill the buffer with 0
    fn zero_padding(&self, read_bytes: usize, buffer_len: usize, buffer: &mut Vec<u8>) {
        if read_bytes < buffer_len {
            let zero: Vec<u8> = vec![0; buffer_len - read_bytes];
            buffer.extend_from_slice(&zero);
        }
    }

    /// Is EOL ?
    fn is_eol(&self) -> bool {
        self.pos >= self.inode.get_size()
    }
}

impl Read for FsFile<'_> {
    /// Pull some bytes from this file into the specified buffer,
    /// returning how many bytes were read
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.is_eol() {
            // End of file
            let zero: Vec<u8> = vec![0; buf.len()];
            buf.copy_from_slice(&zero[..]);
            Ok(0)
        } else {
            let buffer_len = buf.len();
            let read_bytes = self.how_many_bytes(buffer_len);
            let block_num = (self.pos / self.inode.get_block_size()) as u64;
            let mut buffer = self.read_file_block(block_num)?;
            self.zero_padding(read_bytes, buffer_len, &mut buffer);
            let block_pos: usize =
                (self.pos - block_num as u64 * self.inode.get_block_size()) as usize;
            // println!("block num: {} read_bytes: {} pos: {}", block_num, read_bytes, pos);
            buf.copy_from_slice(&buffer[block_pos..block_pos + buffer_len]);
            self.pos += read_bytes as u64;
            Ok(read_bytes)
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
