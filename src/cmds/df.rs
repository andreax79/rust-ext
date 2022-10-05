use std::str;
use std::io::{self, Error};
use crate::fs::Ext2Filesystem;
use argparse::{ArgumentParser};
use humansize::{FileSize, file_size_opts as options};

fn parse_args(args: Vec<String>) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Show information about the file system.");
    if let Err(x) = parser.parse(args, &mut io::stdout(), &mut io::stderr()) {
        std::process::exit(x);
    }
}

pub fn df(filename: &str, args: Vec<String>) -> Result<(), Error> {
    let fs = Ext2Filesystem::open(filename)?;
    parse_args(args);
    let size = fs.super_block.s_blocks_count * fs.super_block.get_blocksize() as u32;
    let avail = fs.super_block.s_free_blocks_count * fs.super_block.get_blocksize() as u32;
    let used = size - avail;
    let use_percent = 100.0 * used as f32 / size as f32;
    println!("Filesystem                        Size     Used    Avail Use%");
    println!("{:30} {:8} {:8} {:8} {:.0}%",
        filename,
        size.file_size(options::DECIMAL).unwrap(),
        used.file_size(options::DECIMAL).unwrap(),
        avail.file_size(options::DECIMAL).unwrap(),
        use_percent,
    );
    Ok(())
}
