use lwext4_rs::FsType::Ext4;
use lwext4_rs::{set_debug_mask, BlockDeviceConfig, DebugFlags, DefaultInterface, FsBuilder};
use std::io::Cursor;

fn main() {
    env_logger::init();

    let mut buf = Vec::with_capacity(1024 * 1024 * 3);
    unsafe {
        buf.set_len(1024 * 1024 * 3);
    }
    let mut config = BlockDeviceConfig::default();

    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = 1024 * 1024 * 3;
    config.part_offset = 0;
    config.block_count = config.part_size / bs;
    println!("config: {:#x?}", config);

    let buf = Cursor::new(buf);
    set_debug_mask(DebugFlags::ALL);

    let blk = DefaultInterface::new_device(buf, config);
    let fs = FsBuilder::new()
        .ty(Ext4)
        .journal(true)
        .block_size(1024)
        .label("ext4fs")
        .build(blk)
        .unwrap();
    println!("{:#x?}", fs.fs_info())

    // let blk = fs.take_device();
}
