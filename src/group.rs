use std::mem;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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
}
