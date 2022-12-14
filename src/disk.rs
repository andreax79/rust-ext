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
        block_size: u64,
        block_num: u64,
    },
    BlockDelta {
        block_size: u64,
        base_block_num: u64,
        delta: u64,
    },
}

impl Offset {
    pub fn calc_offset(&self) -> u64 {
        match self {
            Offset::Block {
                block_size,
                block_num,
            } => *block_num as u64 * *block_size,
            Offset::BlockDelta {
                block_size,
                base_block_num,
                delta,
            } => *base_block_num as u64 * *block_size + *delta,
        }
    }
}

pub struct FileDisk {
    file: UnsafeCell<File>,
}

pub trait Disk {
    fn read(&self, size: u64, offset: Offset) -> Result<Vec<u8>, Error>;

    fn calc_offset(&self, block_size: u64, base_block_num: u64, delta: u64) -> u64 {
        base_block_num as u64 * block_size + delta
    }
}

impl FileDisk {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        Ok(Self { file: file.into() })
    }
}

impl Disk for FileDisk {
    fn read(&self, size: u64, offset: Offset) -> Result<Vec<u8>, Error> {
        let offset: u64 = offset.calc_offset();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(size as usize, 0);
        unsafe {
            let mut file = &*self.file.get();
            file.seek(SeekFrom::Start(offset))?;
            let nbytes: usize = file.read(&mut buffer)?;
            if nbytes != size as usize {
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Not enough bytes read {nbytes} < {size}"),
                ))
            } else {
                Ok(buffer)
            }
        }
    }
}

pub struct BlockCache<'a> {
    disk: &'a Box<dyn Disk>,
    block_size: u64,
    cache: HashMap<u64, Vec<u8>>,
}

impl BlockCache<'_> {
    pub fn new(disk: &Box<dyn Disk>, block_size: u64) -> BlockCache {
        let cache: HashMap<u64, Vec<u8>> = HashMap::new();
        BlockCache {
            disk: disk,
            block_size: block_size,
            cache: cache,
        }
    }

    pub fn get_block(&mut self, block_num: u64) -> Result<&Vec<u8>, Error> {
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
