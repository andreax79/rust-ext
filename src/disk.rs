use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;

pub enum Offset {
    Sector {
        block_size: usize,
        sector_num: u32,
    },
    SectorDelta {
        block_size: usize,
        base_sector_num: u32,
        delta: u64,
    },
}

impl Offset {
    pub fn calc_offset(&self) -> u64 {
        match self {
            Offset::Sector {
                block_size,
                sector_num,
            } => *sector_num as u64 * *block_size as u64,
            Offset::SectorDelta {
                block_size,
                base_sector_num,
                delta,
            } => *base_sector_num as u64 * *block_size as u64 + *delta,
        }
    }
}

pub struct Disk {
    file: File,
}

impl Disk {
    pub fn open(filename: &str) -> Self {
        match File::open(filename) {
            Ok(file) => Self { file: file },
            Err(why) => panic!("Error opening file: {why}"),
        }
    }

    pub fn read(&mut self, size: usize, offset: Offset) -> Vec<u8> {
        let offset: u64 = offset.calc_offset();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(size, 0);
        match self.file.seek(SeekFrom::Start(offset)) {
            Ok(r) => r,
            Err(why) => panic!("Error seeking file {offset}: {why}"),
        };
        let nbytes: usize = match self.file.read(&mut buffer) {
            Ok(nbytes) => nbytes,
            Err(why) => panic!("Error reading file: {why}"),
        };
        if nbytes != size {
            panic!("Not enough bytes read {nbytes} < {size}");
        }
        buffer
    }

    pub fn calc_offset(&self, block_size: usize, base_sector_num: u32, delta: u64) -> u64 {
        base_sector_num as u64 * block_size as u64 + delta
    }
}
