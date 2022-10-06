use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::SeekFrom;

#[derive(Debug)]
pub enum Offset {
    Block {
        block_size: usize,
        block_num: u32,
    },
    BlockDelta {
        block_size: usize,
        base_block_num: u32,
        delta: u64,
    },
}

impl Offset {
    pub fn calc_offset(&self) -> u64 {
        match self {
            Offset::Block {
                block_size,
                block_num,
            } => *block_num as u64 * *block_size as u64,
            Offset::BlockDelta {
                block_size,
                base_block_num,
                delta,
            } => *base_block_num as u64 * *block_size as u64 + *delta,
        }
    }
}

pub struct Disk {
    file: File,
}

impl Disk {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        Ok(Self { file: file })
    }

    pub fn read(&mut self, size: usize, offset: Offset) -> Result<Vec<u8>, Error> {
        let offset: u64 = offset.calc_offset();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(size, 0);
        self.file.seek(SeekFrom::Start(offset))?;
        let nbytes: usize = self.file.read(&mut buffer)?;
        if nbytes != size {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Not enough bytes read {nbytes} < {size}",
            ))
        } else {
            Ok(buffer)
        }
    }

    pub fn calc_offset(&self, block_size: usize, base_block_num: u32, delta: u64) -> u64 {
        base_block_num as u64 * block_size as u64 + delta
    }
}
