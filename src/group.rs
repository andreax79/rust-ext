use crate::disk::Disk;
use crate::disk::Offset;
use crate::superblock::Ext2SuperBlock;
use std::io::Error;
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
    pub bg_flags: u16,
    pub bg_exclude_bitmap_lo: u32,    // Exclude bitmap for snapshots
    pub bg_block_bitmap_csum_lo: u16, // crc32c(s_uuid+grp_num+bitmap) LSB
    pub bg_inode_bitmap_csum_lo: u16, // crc32c(s_uuid+grp_num+bitmap) LSB
    pub bg_itable_unused: u16,        // Unused inodes count
    pub bg_checksum: u16,             // crc16(s_uuid+group_num+group_desc)
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
pub struct GroupDesc {
    pub group_num: usize,               // Group number
    pub ext2_group_desc: Ext2GroupDesc, // Ext2 group desc struct
    pub first_inode_num: u32,           // Fist inode in the group
}
impl GroupDesc {
    pub fn new(group_num: usize, buffer: &Vec<u8>, inodes_per_group: u32) -> GroupDesc {
        GroupDesc {
            group_num: group_num,
            ext2_group_desc: Ext2GroupDesc::new(group_num, &buffer),
            first_inode_num: group_num as u32 * inodes_per_group + 1,
        }
    }
}

#[derive(Debug)]
pub struct Ext2BlockGroups {
    block_groups: Vec<GroupDesc>,
    inodes_per_group: u32, // Number of inodes in each block group
}
impl Ext2BlockGroups {
    // Read the Block Groups
    pub fn new(disk: &Disk, super_block: &Ext2SuperBlock) -> Result<Ext2BlockGroups, Error> {
        let size: usize = EXT2_GROUP_DESC_SIZE * super_block.get_groups_count();
        let block_size = super_block.get_blocksize();
        // Read from disk
        let offset = Offset::Block {
            block_size: block_size,
            // If block size is 1024 the Block Group Descriptor Table will begin at block 2,
            // for any other block size, it will begin at block 1
            block_num: if block_size == 1024 { 2 } else { 1 },
        };
        let buffer = disk.read(size, offset)?;
        // Prepare the Ext2GroupDesc instances
        let block_groups: Vec<GroupDesc> = (0..super_block.get_groups_count())
            .into_iter()
            .map(|i| GroupDesc::new(i, &buffer, super_block.s_inodes_per_group))
            .collect();
        let result = Ext2BlockGroups {
            block_groups: block_groups,
            inodes_per_group: super_block.s_inodes_per_group,
        };
        Ok(result)
    }
    // Determine which block group the inode belongs to and return the group
    pub fn get_inode_group(&self, inode_num: u32) -> &GroupDesc {
        &self.block_groups[((inode_num - 1) / self.inodes_per_group) as usize]
    }
}
