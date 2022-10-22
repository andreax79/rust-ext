use std::cell::UnsafeCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
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
    file: UnsafeCell<File>,
}

impl Disk {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        Ok(Self { file: file.into() })
    }

    pub fn read(&self, size: usize, offset: Offset) -> Result<Vec<u8>, Error> {
        let offset: u64 = offset.calc_offset();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(size, 0);
        unsafe {
            let mut file = &*self.file.get();
            file.seek(SeekFrom::Start(offset))?;
            let nbytes: usize = file.read(&mut buffer)?;
            if nbytes != size {
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Not enough bytes read {nbytes} < {size}"),
                ))
            } else {
                Ok(buffer)
            }
        }
    }

    pub fn calc_offset(&self, block_size: usize, base_block_num: u32, delta: u64) -> u64 {
        base_block_num as u64 * block_size as u64 + delta
    }
}

pub struct BlockCache<'a> {
    disk: &'a Disk,
    block_size: usize,
    cache: HashMap<u32, Vec<u8>>,
}

impl BlockCache<'_> {
    pub fn new(disk: &Disk, block_size: usize) -> BlockCache {
        let cache: HashMap<u32, Vec<u8>> = HashMap::new();
        BlockCache {
            disk: disk,
            block_size: block_size,
            cache: cache,
        }
    }

    pub fn get_block(&mut self, block_num: u32) -> Result<&Vec<u8>, Error> {
        match self.cache.entry(block_num) {
            Vacant(entry) => {
                let offset = Offset::Block {
                    block_size: self.block_size,
                    block_num: block_num,
                };
                let data = self.disk.read(self.block_size, offset)?;
                Ok(entry.insert(data))
            }
            Occupied(entry) => Ok(entry.into_mut()),
        }
    }
}
