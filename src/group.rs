use crate::disk::Disk;
use crate::disk::Offset;
use crate::superblock::Ext2SuperBlock;
use std::io::Read;
use std::mem;
use std::slice;

const EXT2_GROUP_DESC_SIZE: usize = mem::size_of::<Ext2GroupDesc>();

// Blocks are divided up into block groups.
// A block group is a contiguous groups of blocks
#[repr(C)]
#[derive(Debug)]
pub struct Ext2GroupDesc {
    pub bg_block_bitmap: u32, // The block which contains the block bitmap for the group.
    pub bg_inode_bitmap: u32, // The block contains the inode bitmap for the group.
    pub bg_inode_table: u32, // The block contains the inode table first block (the starting block of the inode table.).
    pub bg_free_blocks_count: u16, // Number of free blocks in the group.
    pub bg_free_inodes_count: u16, // Number of free inodes in the group.
    pub bg_used_dirs_count: u16, // Number of inodes allocated to the directories.
    pub bg_pad: u16,         // Padding (reserved).
    pub bg_reserved: [u32; 3], // Reserved.
}
impl Ext2GroupDesc {
    pub fn default() -> Ext2GroupDesc {
        let group: Ext2GroupDesc = unsafe { mem::zeroed() };
        group
    }
    pub fn new(group_num: usize, buffer: &Vec<u8>) -> Ext2GroupDesc {
        let mut group: Ext2GroupDesc = unsafe { mem::zeroed() };
        let mut buf =
            &buffer[EXT2_GROUP_DESC_SIZE * group_num..EXT2_GROUP_DESC_SIZE * (group_num + 1)];
        let p = &mut group as *mut _ as *mut u8;
        unsafe {
            let group_slice = slice::from_raw_parts_mut(p, EXT2_GROUP_DESC_SIZE);
            buf.read_exact(group_slice).unwrap();
        }
        group
    }
}

#[derive(Debug)]
pub struct Ext2BlockGroups {
    block_groups: Vec<Ext2GroupDesc>,
    inodes_per_group: u32, // Number of inodes in each block group
}
impl Ext2BlockGroups {
    // Read the Block Groups
    pub fn new(disk: &mut Disk, super_block: &Ext2SuperBlock) -> Ext2BlockGroups {
        let size: usize = EXT2_GROUP_DESC_SIZE * super_block.get_groups_count();
        // Read from disk
        let offset = Offset::Sector {
            block_size: super_block.get_blocksize(),
            sector_num: 2,
        };
        let buffer = disk.read(size, offset);
        // Prepare the Ext2GroupDesc instances
        let mut block_groups = Vec::new();
        for i in 0..super_block.get_groups_count() {
            block_groups.push(Ext2GroupDesc::new(i, &buffer));
        }
        Ext2BlockGroups {
            block_groups: block_groups,
            inodes_per_group: super_block.s_inodes_per_group,
        }
    }
    // Determine which block group the inode belongs to and return the group
    pub fn get_inode_group(&self, inode_num: u32) -> &Ext2GroupDesc {
        &self.block_groups[((inode_num - 1) / self.inodes_per_group) as usize]
    }
}
