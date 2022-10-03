pub mod group;
pub mod inode;
pub mod superblock;

use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::mem;
use std::slice;
use std::str;

use crate::group::Ext2GroupDesc;
use crate::inode::Ext2Inode;
use crate::superblock::Ext2SuperBlock;

// const FILENAME: &str = "sysgng.dsk";
const FILENAME: &str = "root";
const BLOCK_SIZE: usize = 1024;
const EXT2_ROOT_INO: u32 = 2; /* Root inode */
// Constants relative to the data blocks
const EXT2_NDIR_BLOCKS: usize = 12;
const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
const EXT2_DIND_BLOCK: usize = EXT2_IND_BLOCK + 1;
const EXT2_TIND_BLOCK: usize = EXT2_DIND_BLOCK + 1;
const EXT2_N_BLOCKS: usize = EXT2_TIND_BLOCK + 1;

pub struct Disk {
    file: File,
}

impl Disk {
    fn open(filename: &str) -> Self {
        match File::open(filename) {
            Ok(file) => Self { file: file },
            Err(why) => panic!("Error opening file: {why}"),
        }
    }

    fn read(&mut self, size: usize, offset: u64) -> Vec<u8> {
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

    fn calc_offset(&self, block_size: usize, base_sector_num: u32, delta: u64) -> u64 {
        base_sector_num as u64 * block_size as u64 + delta
    }
}

pub struct FS {
    pub disk: Disk,
    pub block_size: usize,
    pub super_block: Ext2SuperBlock,
    pub block_groups: Vec<Ext2GroupDesc>,
}

impl FS {
    fn open(filename: &str) -> FS {
        let mut disk = Disk::open(filename);
        let super_block = Ext2SuperBlock::new(&mut disk);
        let block_groups = super_block.read_block_groups(&mut disk);
        FS {
            disk: disk,
            block_size: BLOCK_SIZE,
            super_block: super_block,
            block_groups: block_groups,
        }
    }
    fn calc_offset(block_size: usize, base_sector_num: u32, delta: u64) -> u64 {
        base_sector_num as u64 * block_size as u64 + delta
    }
    fn read_sector_size(&mut self, sector_num: u32, block_size: usize) -> Vec<u8> {
        let offset = self.disk.calc_offset(block_size, sector_num, 0);
        self.disk.read(block_size, offset)
    }
    fn read_sector(&mut self, sector_num: u32) -> Vec<u8> {
        let offset = self.disk.calc_offset(self.block_size, sector_num, 0);
        self.disk.read(self.block_size, offset)
    }
    fn get_inode_group(&self, inode_num: u32) -> Ext2GroupDesc {
        // Determine which block group the inode belongs to and return the group
        self.block_groups[((inode_num - 1) / self.super_block.s_inodes_per_group) as usize]
    }
    fn read_inode(&mut self, inode_num: u32) -> Ext2Inode {
        let group = self.get_inode_group(inode_num);
        let size: usize = self.super_block.s_inode_size as usize;
        let offset = self.disk.calc_offset(
            self.block_size,
            group.bg_inode_table,
            (inode_num - 1) as u64 * size as u64,
        );
        let buffer = self.disk.read(size, offset);
        let mut inode = Ext2Inode::default();
        let mut buf = buffer.as_slice();
        unsafe {
            let inode_slice = slice::from_raw_parts_mut(&mut inode as *mut _ as *mut u8, size);
            buf.read_exact(inode_slice).unwrap();
        }
        inode
    }
    // fn resolve(&mut self, path: &str) -> Option<u32> {
    //     let block_size = self.block_size;
    //     let mut inode_num = EXT2_ROOT_INO;
    //     for part in path.split("/") {
    //         let inode = self.read_inode(inode_num);
    //         for buffer in self.read_blocks(&inode) {
    //             let mut offset = 0;
    //             while offset < block_size {
    //                 let (dir_entry, rec_len) = DirEntry::read(&buffer, offset);
    //                 offset += rec_len;
    //                 println!("{:#?}", dir_entry);
    //
    //             }
    //         }
    //         println!("{}", part);
    //     }
    //     None
    // }
    // xxx
    fn readdir(&mut self, inode_num: u32) {
        let inode = self.read_inode(inode_num);
        // println!("{inode:#?}");
        let block_size = self.block_size;
        for buffer in self.read_blocks(&inode) {
            let mut offset = 0;
            while offset < block_size {
                let (dir_entry, rec_len) = DirEntry::read(&buffer, offset);
                offset += rec_len;
                println!("{:#?}", dir_entry);
            }
        }
        // let mut buf = buffer.as_slice();
        // let mut inode: u32;
        // buf.read_exact(&mut inode).unwrap();
        // println!("{res:#?} {s}");
    }
    fn read_blocks<'a>(&'a mut self, inode: &'a Ext2Inode) -> ReadBlock<'a> {
        ReadBlock {
            disk: &mut self.disk,
            block_size: self.block_size,
            inode: inode,
            curr: 0,
            indirect_blocks: [None, None, None],
        }
    }
}

struct ReadBlock<'a> {
    disk: &'a mut Disk,
    block_size: usize,
    inode: &'a Ext2Inode,
    curr: usize,
    indirect_blocks: [Option<Vec<u8>>; 3],
}

impl ReadBlock<'_> {
    fn prepare_block_result(&mut self, block_num: u32) -> Option<Vec<u8>> {
        if block_num == 0 {
            None
        } else {
            Some(self.read_sector(block_num))
        }
    }

    fn read_sector(&mut self, sector_num: u32) -> Vec<u8> {
        let offset = self.disk.calc_offset(self.block_size, sector_num, 0);
        self.disk.read(self.block_size, offset)
    }
}

impl Iterator for ReadBlock<'_> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.curr + 1;
        if current < EXT2_NDIR_BLOCKS {
            // Direct blocks
            let block = self.inode.i_block[current];
            self.prepare_block_result(block)
        } else if current < (EXT2_NDIR_BLOCKS + (self.block_size / mem::size_of::<u32>())) {
            // Indirect blocks
            let addr: usize = (current - EXT2_NDIR_BLOCKS) * mem::size_of::<u32>();
            if self.indirect_blocks[0].is_none() {
                let block = self.inode.i_block[EXT2_IND_BLOCK];
                self.indirect_blocks[0] = Some(self.read_sector(block));
            }
            match &self.indirect_blocks[0] {
                Some(block) => {
                    let bytes: [u8; 4] = [
                        block[addr],
                        block[addr + 1],
                        block[addr + 2],
                        block[addr + 3],
                    ];
                    self.prepare_block_result(u32::from_le_bytes(bytes))
                }
                None => None,
            }

        // TODO: indirect 2 and 3
        } else {
            None
        }
    }
}

// struct Inode<'a> {
//     fs: &'a mut FS,
//     ext2_inode: Ext2Inode
// }
//
// impl Inode<'a> {
//     fn read_blocks(&mut self) -> ReadBlock {
//         ReadBlock { inode: &mut self, curr: 0 }
//     }
// }

#[repr(C)]
#[derive(Debug)]
struct Ext2DirEntry {
    inode_num: u32, /* Inode number */
    rec_len: u16,   /* Directory entry length */
    name_len: u8,   /* Name length */
    file_type: u8, /* Type indicator (only if the feature bit for "directory entries have file type byte" is set) */
}
impl Ext2DirEntry {
    fn default() -> Ext2DirEntry {
        let dir: Ext2DirEntry = unsafe { mem::zeroed() };
        dir
    }
}

/* Directory entry */
#[derive(Debug)]
struct DirEntry {
    file_name: String, /* file name */
    inode_num: u32,    /* inode number */
}
impl DirEntry {
    fn read(buffer: &Vec<u8>, offset: usize) -> (DirEntry, usize) {
        let mut ext2_dir_entry = Ext2DirEntry::default();
        let size = mem::size_of::<Ext2DirEntry>();
        unsafe {
            let mut buf = &buffer[offset..offset + size];
            let dir_slice =
                slice::from_raw_parts_mut(&mut ext2_dir_entry as *mut _ as *mut u8, size);
            buf.read_exact(dir_slice).unwrap();
        }
        let name_slice = &buffer[offset + size..offset + size + ext2_dir_entry.name_len as usize];
        let name = match str::from_utf8(name_slice) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        let dir_entry = DirEntry {
            file_name: String::from(name),
            inode_num: ext2_dir_entry.inode_num,
        };
        (dir_entry, ext2_dir_entry.rec_len as usize)
    }
}

#[allow(dead_code)]
fn format_time(time: u32) -> String {
    let naive = NaiveDateTime::from_timestamp(time.into(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn main() {
    // assert_eq!(BLOCK_SIZE, mem::size_of::<Ext2SuperBlock>());

    // // let mut f = File::open(FILENAME);
    // let mut buffer = [0u8; 512];
    //
    // let mut file = match File::open(FILENAME) {
    //     Ok(file) => file,
    //     Err(why) => panic!("Error opening file: {why}"),
    // };
    //
    // let nbytes = match file.read(&mut buffer) {
    //     Ok(nbytes) => nbytes,
    //     Err(why) => panic!("Error reading file: {why}"),
    // };
    // println!("{nbytes}");

    let mut fs = FS::open(FILENAME);
    println!("done");
    println!("done");
    println!("done");
    // fs.read_superblock();
    // fs.read_groups();

    // let super_block = fs.super_block;
    // println!("Read structure: {:#?}", super_block);

    // println!("s_mtime: {}", format_time(super_block.s_mtime));
    // println!("s_wtime: {}", format_time(super_block.s_wtime));
    // println!("s_lastcheck: {}", format_time(super_block.s_lastcheck));
    // println!("version: {}.{}", super_block.s_rev_level, super_block.s_minor_rev_level);
    // println!("{} {}", super_block.s_blocks_count, super_block.s_blocks_per_group);
    // println!("s_groups_count: {}", super_block.s_groups_count());
    // println!("s_inodes_per_group: {}", super_block.s_inodes_per_group);

    // let inode = fs.read_inode(EXT2_ROOT_INO);
    // println!("{inode:#?}");
    //fs.read_(EXT2_ROOT_INO);
    //fs.read_(12); // bin
    fs.readdir(525); // etc
                     // let i = fs.resolve("etc/ppp");
                     // println!("done {i:#?}");

    // let inode = fs.read_inode(13);
    // for block in fs.read_blocks(&inode) {
    //     println!("{:#?}", block);
    // }
    // println!("{inode:#?}");
    println!("done");

    // let group_desc_size = mem::size_of::<Ext2GroupDesc>();
    // println!("{group_desc_size}");

    // let mut buffer = [0; 512];

    // let mut handle = f.take(512);
    // handle.read(&mut buffer);
}
