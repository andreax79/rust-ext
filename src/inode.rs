use std::mem;

// Constants relative to the data blocks
pub const EXT2_NDIR_BLOCKS: usize = 12;
pub const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
pub const EXT2_DIND_BLOCK: usize = EXT2_IND_BLOCK + 1;
pub const EXT2_TIND_BLOCK: usize = EXT2_DIND_BLOCK + 1;
pub const EXT2_N_BLOCKS: usize = EXT2_TIND_BLOCK + 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Ext2Inode {
    pub i_mode: u16,        /* File mode */
    pub i_uid: u16,         /* Low 16 bits of Owner Uid */
    pub i_size: u32,        /* Size in bytes */
    pub i_atime: u32,       /* Access time */
    pub i_ctime: u32,       /* Creation time */
    pub i_mtime: u32,       /* Modification time */
    pub i_dtime: u32,       /* Deletion Time */
    pub i_gid: u16,         /* Low 16 bits of Group Id */
    pub i_links_count: u16, /* Links count */
    pub i_blocks: u32, /* Blocks count - Count of disk sectors (not Ext2 blocks) in use by this inode */
    pub i_flags: u32,  /* File flags */
    pub l_i_reserved1: u32,
    pub i_block: [u32; EXT2_N_BLOCKS], /* Pointers to blocks (12) +
                                       1 Singly Indirect Block Pointer (Points to a block that is a list of block pointers to data)
                                       1 Doubly Indirect Block Pointer (Points to a block that is a list of block pointers to Singly Indirect Blocks)
                                       1 Triply Indirect Block Pointer (Points to a block that is a list of block pointers to Doubly Indirect Blocks) */
    pub i_generation: u32, /* File version (for NFS) */
    pub i_file_acl: u32,   /* File ACL */
    pub i_dir_acl: u32,    /* Directory ACL */
    pub i_faddr: u32,      /* Fragment address */
    pub l_i_frag: u8,      /* Fragment number */
    pub l_i_fsize: u8,     /* Fragment size */
    pub i_pad1: u16,
    pub l_i_uid_high: u16, /* these 2 fields    */
    pub l_i_gid_high: u16, /* were reserved2[0] */
    pub l_i_reserved2: u32,
}

impl Ext2Inode {
    pub fn default() -> Ext2Inode {
        let ionode: Ext2Inode = unsafe { mem::zeroed() };
        ionode
    }
}
