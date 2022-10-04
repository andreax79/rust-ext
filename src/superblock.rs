use crate::disk::Offset;
use crate::Disk;
use std::io::Read;
use std::mem;
use std::slice;

const SUPER_BLOCK_SIZE: usize = 1024;
const SUPER_BLOCK: u32 = 1;

#[repr(C)]
#[derive(Debug)]
pub struct Ext2SuperBlock {
    pub s_inodes_count: u32,      // Total number of inodes in file system
    pub s_blocks_count: u32,      // Total number of blocks in file system
    pub s_r_blocks_count: u32,    // Number of blocks reserved for superuser
    pub s_free_blocks_count: u32, // Total number of unallocated blocks
    pub s_free_inodes_count: u32, // Total number of unallocated inodes
    pub s_first_data_block: u32,  // Block number of the block containing the superblock
    pub s_log_block_size: u32,
    pub s_log_frag_size: u32,
    pub s_blocks_per_group: u32, // Number of blocks in each block group
    pub s_frags_per_group: u32,  // Number of fragments in each block group
    pub s_inodes_per_group: u32, // Number of inodes in each block group
    pub s_mtime: u32,            // Last mount time
    pub s_wtime: u32,            // Last written time
    pub s_mnt_count: u16, // Number of times the volume has been mounted since its last consistency check
    pub s_max_mnt_count: u16, // umber of mounts allowed before a consistency check must be done
    pub s_magic: u16, // Ext2 signature (0xef53), used to help confirm the presence of Ext2 on a volume
    pub s_state: u16, // File system state
    pub s_pad: u16,   // What to do when an error is detected
    pub s_minor_rev_level: u16, // Minor portion of version
    pub s_lastcheck: u32, // Time of last consistency check
    pub s_checkinterval: u32, // Interval (in POSIX time) between forced consistency checks
    pub s_creator_os: u32, // Operating system ID from which the filesystem on this volume was created
    pub s_rev_level: u32,  // Major portion of version
    pub s_def_resuid: u16, // User ID that can use reserved blocks
    pub s_def_regid: u16,  // Group ID that can use reserved blocks
    pub s_first_ino: u32,
    pub s_inode_size: u16,
    pub s_block_group_nr: u16,
    pub s_feature_compat: u32,
    pub s_feature_incompat: u32,
    pub s_feature_ro_compat: u32,
    pub s_uuid: [u8; 16],
    pub s_volume_name: [u8; 16],
    pub s_last_mounted: [u8; 64],
    pub s_algorithm_usage_bitmap: u32,
    pub s_prealloc_blocks: u8,
    pub s_prealloc_dir_blocks: u8,
    pub s_padding1: u16,
    s_reserved: [u32; 204],
}

impl Ext2SuperBlock {
    pub fn default() -> Ext2SuperBlock {
        let super_block: Ext2SuperBlock = unsafe { mem::zeroed() };
        super_block
    }
    // Number of groups in the fs
    pub fn get_groups_count(&self) -> usize {
        (self.s_blocks_count as f64 / self.s_blocks_per_group as f64).ceil() as usize
    }
    // Get block size
    pub fn get_blocksize(&self) -> usize {
        (1024 << self.s_log_block_size) as usize
    }
    // Read the Superblock
    pub fn new(disk: &mut Disk) -> Ext2SuperBlock {
        let mut super_block: Ext2SuperBlock = unsafe { mem::zeroed() };
        assert_eq!(mem::size_of::<Ext2SuperBlock>(), SUPER_BLOCK_SIZE);
        let offset = Offset::Sector {
            block_size: SUPER_BLOCK_SIZE,
            sector_num: SUPER_BLOCK,
        };
        let buffer = disk.read(SUPER_BLOCK_SIZE, offset);
        let p = &mut super_block as *mut _ as *mut u8;
        unsafe {
            let block_slice = slice::from_raw_parts_mut(p, SUPER_BLOCK_SIZE);
            match buffer.as_slice().read_exact(block_slice) {
                Ok(r) => r,
                Err(why) => panic!("Error reading file: {why}"),
            };
        }
        // Check ext2 signature
        assert_eq!(0xef53, super_block.s_magic);
        super_block
    }
}
