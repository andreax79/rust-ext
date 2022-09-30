use std::io::prelude::*;
use std::io::Read;
use std::fs::File;
use std::io::SeekFrom;
use std::mem;
use std::slice;

const FILENAME: &str = "root";
const BLOCK_SIZE: usize = 1024;

struct FS {
    file: File,
    block_size: usize,
}

impl FS {
    fn open(filename: &str) -> FS{
        let file = match File::open(filename) {
            Ok(file) => file,
            Err(why) => panic!("Error opening file: {why}"),
        };
        FS{ file: file, block_size: BLOCK_SIZE }
    }
    fn read_sector(&mut self, sector_num: u64) -> (Vec<u8>, usize) {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(self.block_size, 0);
        match self.file.seek(SeekFrom::Start(sector_num * self.block_size as u64)) {
            Ok(r) => r,
            Err(why) => panic!("Error seeking file: {why}")
        };
        let nbytes: usize = match self.file.read(&mut buffer) {
            Ok(nbytes) => nbytes,
            Err(why) => panic!("Error reading file: {why}")
        };
        (buffer, nbytes)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Ext2SuperBlock {
    s_inodes_count: u32,
    s_blocks_count: u32,
    s_r_blocks_count: u32,
    s_free_blocks_count: u32,
    s_free_inodes_count: u32,
    s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_frag_size: u32,
    s_blocks_per_group: u32,
    s_frags_per_group: u32,
    s_inodes_per_group: u32,
    s_mtime: u32,
    s_wtime: u16,
    s_mnt_count: u16,
    s_max_mnt_coun: u16,
    s_magi: u16,
    s_stat: u16,
    s_pa: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterva: u32,
    s_creator_o: u32,
    s_rev_leve: u32,
    s_def_resuid: u16,
    s_def_regid: u16,
    s_first_ino: u32,
    s_inode_size: u16,
    s_block_group_nr: u16,
    s_feature_compat: u32,
    s_feature_incompat: u32,
    s_feature_ro_compat: u32,
    s_uuid: [u8; 16],
    s_volume_name: [u8; 16],
    s_last_mounted: [u8; 64],
    s_algorithm_usage_bitmap: u32,
    s_prealloc_blocks: u8,
    s_prealloc_dir_blocks: u8,
    s_padding1: u16,
    s_reserved: [u32; 204]
}


fn main() {
    assert_eq!(BLOCK_SIZE, mem::size_of::<Ext2SuperBlock>());

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
    let (buffer, nbytes) = fs.read_sector(1);
    println!("{nbytes}");

    let mut super_block: Ext2SuperBlock = unsafe { mem::zeroed() };
    unsafe {
        let block_slice = slice::from_raw_parts_mut(&mut super_block as *mut _ as *mut u8, BLOCK_SIZE);
        buffer.read_exact(block_slice).unwrap();
    }

    println!("Read structure: {:#?}", super_block);

    // let mut buffer = [0; 512];

    // let mut handle = f.take(512);
    // handle.read(&mut buffer);

}
