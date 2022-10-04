use crate::dir::DirEntry;
use crate::disk::Offset;
use crate::group::Ext2BlockGroups;
use crate::superblock::Ext2SuperBlock;
use crate::Disk;
use std::collections::BTreeMap;
use std::io::Read;
use std::mem;
use std::slice;

// Constants relative to the data blocks
pub const EXT2_NDIR_BLOCKS: usize = 12;
pub const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
pub const EXT2_DIND_BLOCK: usize = EXT2_IND_BLOCK + 1;
pub const EXT2_TIND_BLOCK: usize = EXT2_DIND_BLOCK + 1;
pub const EXT2_N_BLOCKS: usize = EXT2_TIND_BLOCK + 1;

// Filetype information as used in inodes
// pub const FILETYPE_INO_MASK	0170000
pub const FILETYPE_INO_REG: u16 = 32768;
pub const FILETYPE_INO_DIRECTORY: u16 = 16384;
pub const FILETYPE_INO_SYMLINK: u16 = 40960;

#[repr(C)]
#[derive(Debug)]
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

#[derive(Debug)]
pub struct Inode {
    pub inode_num: u32,        // Inode number
    pub ext2_inode: Ext2Inode, // Ext2 inode struct
    block_size: usize,         // Block size
}

impl Inode {
    pub fn new(
        disk: &mut Disk,
        super_block: &Ext2SuperBlock,
        block_groups: &Ext2BlockGroups,
        inode_num: u32,
    ) -> Inode {
        let group = block_groups.get_inode_group(inode_num);
        let size: usize = super_block.s_inode_size as usize;
        let block_size = super_block.get_blocksize();
        let offset = Offset::SectorDelta {
            block_size: block_size,
            base_sector_num: group.bg_inode_table,
            delta: (inode_num - 1) as u64 * size as u64,
        };
        let buffer = disk.read(size, offset);
        let mut inode = Ext2Inode::default();
        let mut buf = buffer.as_slice();
        let p = &mut inode as *mut _ as *mut u8;
        unsafe {
            let inode_slice = slice::from_raw_parts_mut(p, size);
            buf.read_exact(inode_slice).unwrap();
        }
        Inode {
            inode_num: inode_num,
            ext2_inode: inode,
            block_size: block_size,
        }
    }

    // Read blocks
    pub fn read_blocks<'a>(&self, disk: &'a mut Disk) -> ReadBlock<'a> {
        ReadBlock {
            disk: disk,
            block_size: self.block_size,
            i_block: self.ext2_inode.i_block,
            indirect_blocks: [None, None, None],
            curr: 0,
        }
    }

    // Resolve a child
    pub fn get_child(
        &self,
        disk: &mut Disk,
        super_block: &Ext2SuperBlock,
        block_groups: &Ext2BlockGroups,
        name: &str,
    ) -> Option<Inode> {
        match self.readdir(disk) {
            Ok(entries) => match entries.get(name) {
                Some(dir_entry) => Some(Inode::new(
                    disk,
                    super_block,
                    block_groups,
                    dir_entry.inode_num,
                )),
                None => None,
            },
            Err(_) => None,
        }
    }

    // Read a directory
    pub fn readdir(&self, disk: &mut Disk) -> Result<BTreeMap<String, DirEntry>, &'static str> {
        if !self.is_dir() {
            Err("Not a directory")
        } else {
            let mut entries = BTreeMap::new();
            // Iterate over blocks
            for buffer in self.read_blocks(disk) {
                let mut offset = 0;
                // Iterate over block directory entries
                while offset < self.block_size {
                    let (dir_entry, rec_len) = DirEntry::read(&buffer, offset);
                    offset += rec_len;
                    entries.insert(dir_entry.file_name.clone(), dir_entry);
                }
            }
            Ok(entries)
        }
    }

    // Tests whether this inode is a directory
    pub fn is_dir(&self) -> bool {
        self.ext2_inode.i_mode & FILETYPE_INO_DIRECTORY == FILETYPE_INO_DIRECTORY
    }

    // Tests whether this inode is a regular file
    pub fn is_file(&self) -> bool {
        self.ext2_inode.i_mode & FILETYPE_INO_REG == FILETYPE_INO_REG
    }

    // Tests whether this inode is a symbolic link
    pub fn is_symlink(&self) -> bool {
        self.ext2_inode.i_mode & FILETYPE_INO_SYMLINK == FILETYPE_INO_SYMLINK
    }
}

pub struct ReadBlock<'a> {
    disk: &'a mut Disk,
    block_size: usize,
    i_block: [u32; EXT2_N_BLOCKS],
    indirect_blocks: [Option<Vec<u8>>; 3],
    curr: usize,
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
        let offset = Offset::Sector {
            block_size: self.block_size,
            sector_num: sector_num,
        };
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
            let block = self.i_block[current];
            self.prepare_block_result(block)
        } else if current < (EXT2_NDIR_BLOCKS + (self.block_size / mem::size_of::<u32>())) {
            // Indirect blocks
            let addr: usize = (current - EXT2_NDIR_BLOCKS) * mem::size_of::<u32>();
            if self.indirect_blocks[0].is_none() {
                let block = self.i_block[EXT2_IND_BLOCK];
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
