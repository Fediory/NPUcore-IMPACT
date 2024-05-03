use embedded_io::Read;
use lwext4_rs::set_debug_mask;
use lwext4_rs::FileSystem;
use lwext4_rs::{BlockDeviceConfig, DefaultInterface, MountHandle, RegisterHandle};
use lwext4_rs::{DebugFlags, MetaDataExt};
use std::fs::OpenOptions;

fn main() {
    env_logger::init();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("ext_images/ext_image")
        .unwrap();
    let mut config = BlockDeviceConfig::default();

    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = file.metadata().unwrap().len();
    config.part_offset = 0;
    config.block_count = config.part_size / bs;

    println!("config: {:#x?}", config);

    set_debug_mask(DebugFlags::ALL);
    let blk = DefaultInterface::new_device(file, config);
    let register_handler = RegisterHandle::register(blk, "ext4fs".to_string()).unwrap();
    let mount_handler =
        MountHandle::mount(register_handler, "/mp/".to_string(), true, false).unwrap();
    let fs = FileSystem::new(mount_handler).unwrap();

    let stats = fs.mount_handle().stats().unwrap();
    println!("stats: {:#x?}", stats);

    let read_dir = fs.readdir("/mp/").unwrap();
    for entry in read_dir {
        println!("{:?}", entry);
    }

    let res = fs.remove_file("/mp/1.txt");
    println!("{:?}", res);

    let mut file = fs.file_builder().read(true).open("/mp/f1").unwrap();
    let mut buf = [0u8; 512];
    let res = file.read(&mut buf);
    println!("{:?}", res);
    let str = std::str::from_utf8(&buf).unwrap();
    println!("{}", str);

    let meta = fs.metadata("/mp/f1").unwrap();
    println!("{:#x?}", meta);
    println!("size: {}, blocks: {}", meta.len(), meta.blocks());

    let file = fs
        .file_builder()
        .read(true)
        .write(true)
        .create(true)
        .open("/mp/1.txt");
    if let Ok(file) = file {
        let res = file.metadata();
        println!("{:#x?}", res);
    }
    // drop(mount_handler);
}
