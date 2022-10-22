use crate::dir::DirEntry;
use crate::disk::{BlockCache, Disk, Offset};
use crate::group::Ext2BlockGroups;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::mem;
use std::slice;
use std::str;

// Constants relative to the data blocks
pub const EXT2_NDIR_BLOCKS: usize = 12;
pub const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
pub const EXT2_DOUBLY_IND_BLOCK: usize = EXT2_IND_BLOCK + 1;
pub const EXT2_TRIPLY_IND_BLOCK: usize = EXT2_DOUBLY_IND_BLOCK + 1;
pub const EXT2_N_BLOCKS: usize = EXT2_TRIPLY_IND_BLOCK + 1;
pub const I_BLOCKS_SIZE: usize = EXT2_N_BLOCKS * 4;

#[repr(C)]
#[derive(Debug)]
pub struct Ext2Inode {
    pub i_mode: u16,        /* File mode */
    pub i_uid: u16,         /* Low 16 bits of Owner Uid */
    pub i_size: u32,        /* Lower 32 bits of size in bytes */
    pub i_atime: u32,       /* Access time */
    pub i_ctime: u32,       /* Creation time */
    pub i_mtime: u32,       /* Modification time */
    pub i_dtime: u32,       /* Deletion Time */
    pub i_gid: u16,         /* Low 16 bits of Group Id */
    pub i_links_count: u16, /* Links count */
    pub i_blocks: u32,      /* Count of disk sectors (not Ext2 blocks) in use by this inode */
    pub i_flags: u32,       /* File flags */
    pub l_i_reserved1: u32,
    pub i_block: [u32; EXT2_N_BLOCKS], /* Pointers to blocks (12) +
                                       1 Singly Indirect Block Pointer (Points to a block that is a list of block pointers to data)
                                       1 Doubly Indirect Block Pointer (Points to a block that is a list of block pointers to Singly Indirect Blocks)
                                       1 Triply Indirect Block Pointer (Points to a block that is a list of block pointers to Doubly Indirect Blocks) */
    pub i_generation: u32, /* File version (for NFS) */
    pub i_file_acl: u32,   /* File ACL */
    pub i_size_high: u32,
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
    pub fn size(&self) -> u64 {
        // Calculate the size in bytes
        if unix_mode::is_file(self.i_mode as u32) {
            self.i_size as u64 | ((self.i_size_high as u64) << 32)
        } else {
            self.i_size as u64
        }
    }
}

#[derive(Debug)]
pub struct Inode {
    pub inode_num: u32,         // Inode number
    pub ext2_inode: Ext2Inode,  // Ext2 inode struct
    pub inode_size: usize,      // Inode size
    pub block_size: usize,      // Block size
    pub size: u64,              // Size in bytes
    pub data_blocks_count: u32, // Number of data blocks
}

impl Inode {
    pub fn new(
        disk: &Disk,
        inode_size: usize,
        block_size: usize,
        block_groups: &Ext2BlockGroups,
        inode_num: u32,
    ) -> Result<Inode, Error> {
        // Determinate the block group
        let group = block_groups.get_inode_group(inode_num);
        // Calculate the offset
        let offset = Offset::BlockDelta {
            block_size: block_size,
            base_block_num: group.ext2_group_desc.bg_inode_table,
            delta: (inode_num - group.first_inode_num) as u64 * inode_size as u64,
        };
        // Read the inode from the disk
        let buffer = disk.read(inode_size, offset)?;
        let mut inode = Ext2Inode::default();
        let mut buf = buffer.as_slice();
        let p = &mut inode as *mut _ as *mut u8;
        unsafe {
            let inode_slice = slice::from_raw_parts_mut(p, inode_size);
            buf.read_exact(inode_slice).unwrap();
        }
        // Calculate the size
        let size = inode.size();
        // Calculate the number of data blocks
        let data_blocks_count: u32 = (size as f64 / block_size as f64).ceil() as u32;
        Ok(Inode {
            inode_num: inode_num,
            ext2_inode: inode,
            inode_size: inode_size,
            block_size: block_size,
            size: size,
            data_blocks_count: data_blocks_count,
        })
    }

    // Read blocks iterator
    pub fn read_blocks_iter<'a>(&'a self, disk: &'a Disk) -> Result<ReadBlock<'a>, Error> {
        Ok(ReadBlock {
            disk: disk,
            block_size: self.block_size,
            blocks: self.get_blocks_iter(disk)?,
        })
    }

    // Read file content
    pub fn read(&self, disk: &Disk) -> Result<Vec<u8>, Error> {
        let mut buffer: Vec<u8> = Vec::new();
        for block in self.read_blocks_iter(disk)? {
            buffer.extend(&block?);
        }
        Ok(buffer)
    }

    // Block numbers iterator
    pub fn get_blocks_iter<'a>(&'a self, disk: &'a Disk) -> Result<ReadBlockNum<'a>, Error> {
        Ok(ReadBlockNum::new(
            disk,
            &self.ext2_inode.i_block,
            self.block_size,
            self.data_blocks_count,
        ))
    }

    // Block numbers
    pub fn get_blocks(&self, disk: &Disk) -> Result<Vec<u32>, Error> {
        match self.get_blocks_iter(disk) {
            Ok(iterator) => iterator.collect::<Result<Vec<_>, _>>(),
            Err(x) => Err(x),
        }
    }

    // Resolve a child by name - return the child's inode
    pub fn get_child(
        &self,
        disk: &Disk,
        block_groups: &Ext2BlockGroups,
        name: &str,
    ) -> Option<Inode> {
        match self.readdir(disk) {
            Ok(entries) => match entries.get(name) {
                Some(dir_entry) => Some(
                    Inode::new(
                        disk,
                        self.inode_size,
                        self.block_size,
                        block_groups,
                        dir_entry.inode_num,
                    )
                    .ok()?,
                ),
                None => None,
            },
            Err(_) => None,
        }
    }

    // Read a directory
    pub fn readdir(&self, disk: &Disk) -> Result<BTreeMap<String, DirEntry>, Error> {
        if !self.is_dir() {
            Err(Error::new(ErrorKind::InvalidInput, "Not a directory"))
            // Err(Error::new(ErrorKind::NotADirectory, "Not a directory"))
        } else {
            let mut entries = BTreeMap::new();
            // Iterate over blocks
            for buffer in self.read_blocks_iter(disk)? {
                let buffer = buffer?;
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

    // Read value of a symbolic link
    pub fn readlink(&self, disk: &Disk) -> Result<String, Error> {
        // The target of a symbolic link is stored in the inode
        // if it is less than 60 bytes long.
        if self.size <= I_BLOCKS_SIZE as u64 {
            let buffer: [u8; I_BLOCKS_SIZE] = unsafe { mem::transmute(self.ext2_inode.i_block) };
            let target = &buffer[0..self.size as usize];
            match str::from_utf8(target) {
                Ok(result) => Ok(String::from(result)),
                Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
            }
        } else {
            match String::from_utf8(self.read(disk)?) {
                Ok(result) => Ok(result),
                Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
            }
        }
    }

    // Tests whether this inode is a directory
    pub fn is_dir(&self) -> bool {
        return unix_mode::is_dir(self.ext2_inode.i_mode as u32);
    }

    // Tests whether this inode is a regular file
    pub fn is_file(&self) -> bool {
        return unix_mode::is_file(self.ext2_inode.i_mode as u32);
    }

    // Tests whether this inode is a symbolic link
    pub fn is_symlink(&self) -> bool {
        return unix_mode::is_symlink(self.ext2_inode.i_mode as u32);
    }
}

pub struct ReadBlockNum<'a> {
    blocks_per_block: u32, // number of block number (u32) in a block
    i_block: &'a [u32; EXT2_N_BLOCKS],
    data_blocks_count: u32,
    cache: BlockCache<'a>,
    first_indirect_block: u32,
    first_doubly_indirect_block: u32,
    first_triply_indirect_block: u32,
    curr: u32,
}

impl ReadBlockNum<'_> {
    pub fn new<'a>(
        disk: &'a Disk,
        i_block: &'a [u32; EXT2_N_BLOCKS],
        block_size: usize,
        data_blocks_count: u32,
    ) -> ReadBlockNum<'a> {
        let blocks_per_block = (block_size / mem::size_of::<u32>()) as u32;
        ReadBlockNum {
            blocks_per_block: blocks_per_block,
            i_block: i_block,
            data_blocks_count: data_blocks_count,
            cache: BlockCache::new(disk, block_size),
            first_indirect_block: EXT2_NDIR_BLOCKS as u32,
            first_doubly_indirect_block: EXT2_NDIR_BLOCKS as u32 + blocks_per_block,
            first_triply_indirect_block: EXT2_NDIR_BLOCKS as u32
                + blocks_per_block
                + (blocks_per_block * blocks_per_block),
            curr: 0,
        }
    }

    fn get_direct_block(&self, i: u32) -> Result<u32, Error> {
        // Get direct block
        Ok(self.i_block[i as usize])
    }

    fn get_indirect_block(&mut self, i: u32, indirect_block_num: u32) -> Result<u32, Error> {
        // Get singly indirect block
        let indirect_blocks = self.cache.get_block(indirect_block_num)?;
        let addr: usize = i as usize * mem::size_of::<u32>();
        let bytes: [u8; 4] = indirect_blocks[addr..addr + 4]
            .try_into()
            .expect("incorrect length");
        Ok(u32::from_le_bytes(bytes))
    }

    fn get_doubly_indirect_block(
        &mut self,
        i: u32,
        doubly_indirect_block_num: u32,
    ) -> Result<u32, Error> {
        // Get doubly indirect block
        let indirect_block_num_i = i / self.blocks_per_block;
        let indirect_block_num =
            self.get_indirect_block(indirect_block_num_i, doubly_indirect_block_num)?;
        let i = i - indirect_block_num_i * self.blocks_per_block;
        self.get_indirect_block(i, indirect_block_num)
    }

    fn get_triply_indirect_block(
        &mut self,
        i: u32,
        triply_indirect_block_num: u32,
    ) -> Result<u32, Error> {
        // Get triply indirect block
        let doubly_indirect_block_num_i = i / self.blocks_per_block / self.blocks_per_block;
        let doubly_indirect_block_num =
            self.get_indirect_block(doubly_indirect_block_num_i, triply_indirect_block_num)?;
        let i = i - doubly_indirect_block_num_i * self.blocks_per_block * self.blocks_per_block;
        self.get_doubly_indirect_block(i, doubly_indirect_block_num)
    }
}

impl Iterator for ReadBlockNum<'_> {
    // Everything is wrapped in a Result so that we can pass IO errors the caller
    type Item = Result<u32, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.curr;
        if self.curr >= self.data_blocks_count {
            None
        } else {
            self.curr = self.curr + 1;
            if i < self.first_indirect_block {
                Some(self.get_direct_block(i))
            } else if i < self.first_doubly_indirect_block {
                let i = i - self.first_indirect_block;
                let indirect_block_num: u32 = self.i_block[EXT2_IND_BLOCK];
                Some(self.get_indirect_block(i, indirect_block_num))
            } else if i < self.first_triply_indirect_block {
                let i = i - self.first_doubly_indirect_block;
                let doubly_indirect_block_num: u32 = self.i_block[EXT2_DOUBLY_IND_BLOCK];
                Some(self.get_doubly_indirect_block(i, doubly_indirect_block_num))
            } else {
                let i = i - self.first_triply_indirect_block;
                let triply_indirect_block_num: u32 = self.i_block[EXT2_TRIPLY_IND_BLOCK];
                Some(self.get_triply_indirect_block(i, triply_indirect_block_num))
            }
        }
    }
}

pub struct ReadBlock<'a> {
    disk: &'a Disk,
    block_size: usize,
    blocks: ReadBlockNum<'a>,
}

impl ReadBlock<'_> {
    fn prepare_block_result(&mut self, block_num: u32) -> Option<Result<Vec<u8>, Error>> {
        if block_num == 0 {
            None
        } else {
            Some(self.read_block(block_num))
        }
    }

    fn read_block(&mut self, block_num: u32) -> Result<Vec<u8>, Error> {
        let offset = Offset::Block {
            block_size: self.block_size,
            block_num: block_num,
        };
        self.disk.read(self.block_size, offset)
    }
}

impl Iterator for ReadBlock<'_> {
    // Everything is wrapped in a Result so that we can pass IO errors the caller
    type Item = Result<Vec<u8>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.blocks.next() {
            Some(block) => self.prepare_block_result(block.ok()?),
            None => None,
        }
    }
}
