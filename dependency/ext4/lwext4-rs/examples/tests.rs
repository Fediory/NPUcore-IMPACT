use embedded_io::{Read, Write};
use lwext4_rs::FsType::Ext4;
use lwext4_rs::*;
use std::io::Cursor;

fn main() -> Result<()> {
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
    println!("{:#x?}", fs.fs_info());

    let blk = fs.take_device();
    let register_handler = RegisterHandle::register(blk, "ext4fs".to_string()).unwrap();
    let mount_handler =
        MountHandle::mount(register_handler, "/mp/".to_string(), true, false).unwrap();
    let mut fs = FileSystem::new(mount_handler).unwrap();

    test_cleanup(&mut fs)?;
    test_dir_ls(&mut fs, "/mp/")?;
    test_dir_test(&mut fs, 10)?;
    test_file_test(&mut fs, 1024 * 64, 16)?; // 1GB

    test_xattr(&mut fs)?;
    test_mknod(&mut fs)?;
    test_link(&mut fs)?;
    test_rename(&mut fs)?;
    test_link_rename(&mut fs)?;

    test_cleanup(&mut fs)?;
    test_dir_ls(&mut fs, "/mp/")?;
    Ok(())
}

fn test_cleanup<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_cleanup:");
    let err = |e: Result<()>| match e {
        Ok(_) => {}
        Err(Error::NoEntry) => {}
        _ => {
            println!("remove file/dir error: rc = {:?}", e);
        }
    };
    err(fs.remove_file("/mp/hello.txt"));
    err(fs.remove_file("/mp/test1"));
    err(fs.remove_file("/mp/hello1.txt"));
    err(fs.remove_file("/mp/hello2.txt"));
    err(fs.remove_dir("/mp/dir1"));
    err(fs.remove_dir_all("/mp/dir2"));
    err(fs.remove_file("/mp/hello-rename.txt"));
    err(fs.remove_file("/mp/fifo"));
    err(fs.remove_file("/mp/blk"));
    println!("test_cleanup Pass");
    Ok(())
}

fn test_dir_ls<T: BlockDeviceInterface>(fs: &mut FileSystem<T>, path: &str) -> Result<()> {
    println!("ls {}:", path);
    let dir = fs.readdir(path)?;
    for entry in dir {
        println!("{:?}", entry);
    }
    println!("ls {} Pass", path);
    Ok(())
}

fn test_dir_test<T: BlockDeviceInterface>(fs: &mut FileSystem<T>, len: usize) -> Result<()> {
    println!("test_dir_test: {}", len);
    println!("directory create: /mp/dir1");
    fs.create_dir("/mp/dir1").map_err(|e| {
        println!("ext4_dir_create: rc = {:?}", e);
        e
    })?;
    println!("add files to: /mp/dir1");
    for i in 0..len {
        let mut path = String::from("/mp/dir1/");
        let file_name = format!("f{}", i);
        path.push_str(&file_name);
        fs.file_builder()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|e| {
                println!("ext4_fopen: rc = {:?}", e);
                e
            })?;
    }
    fs.create_dir("/mp/dir1/dir2").map_err(|e| {
        println!("ext4_dir_create: rc = {:?}", e);
        e
    })?;
    fs.create_dir_all("/mp/dir2").map_err(|e| {
        println!("ext4_dir_create: rc = {:?}", e);
        e
    })?;
    test_dir_ls(fs, "/mp/dir1/")?;
    println!("test_dir_test Pass");
    Ok(())
}

fn test_file_test<T: BlockDeviceInterface>(
    fs: &mut FileSystem<T>,
    size: usize,
    count: usize,
) -> Result<()> {
    println!("test_file_test: rw size: {}, rw count: {}", size, count);
    let mut file = fs
        .file_builder()
        .read(true)
        .write(true)
        .create(true)
        .open("/mp/hello.txt")?;
    file.write(b"hello world")?;
    let mut file = fs
        .file_builder()
        .read(true)
        .write(true)
        .create(true)
        .open("/mp/test1")
        .map_err(|e| {
            println!("ext4_fopen: rc = {:?}", e);
            e
        })?;
    println!("ext4_write {} bytes", size * count);
    let mut buf = vec![0u8; size];

    let mut w_count = 0;
    for i in 0..count {
        w_count += 1;
        buf.fill(i as u8 + b'0');
        let res = file.write(&buf);
        match res {
            Ok(r) => {
                if r != size {
                    break;
                }
            }
            _ => {
                break;
            }
        }
    }
    if w_count != count {
        println!("ext4_write error: rc = {:?}", w_count);
        return Err(Error::Io);
    }
    drop(file);

    let mut file = fs
        .file_builder()
        .read(true)
        .open("/mp/test1")
        .map_err(|e| {
            println!("ext4_fopen: rc = {:?}", e);
            e
        })?;
    println!("ext4_read {} bytes", size * count);
    let mut buf = vec![0u8; size];
    let mut r_count = 0;
    for i in 0..count {
        r_count += 1;
        let res = file.read(&mut buf);
        match res {
            Ok(r) => {
                if r != size {
                    break;
                }
            }
            _ => break,
        }
        // verify
        if let Some(_) = buf.iter().find(|&&x| x != (i as u8 + b'0')) {
            break;
        }
    }
    if r_count != count {
        println!("ext4_read error: rc = {:?}", r_count);
        return Err(Error::Io);
    }
    println!("test_file_test Pass");
    Ok(())
}

fn test_xattr<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_xattr");
    let res = fs.get_xattr("/mp/hello.txt", "user.test");
    println!("getxattr: {:?}", res);
    match res {
        Err(_) => {}
        _ => {
            println!("getxattr error: rc = {:?}", res);
            return Err(Error::InvalidArgument);
        }
    }
    fs.set_xattr("/mp/hello.txt", "user.test", b"hello world")
        .map_err(|e| {
            println!("set_xattr error: rc = {:?}", e);
            e
        })?;
    println!("set_xattr: user.test = hello world");
    let res = fs.get_xattr("/mp/hello.txt", "user.test");
    match res {
        Ok(ref r) => {
            if r != b"hello world" {
                println!("getxattr error: rc = {:?}", core::str::from_utf8(&r));
                return Err(Error::InvalidArgument);
            }
            println!("getxattr: {:?}", core::str::from_utf8(&r));
        }
        _ => {
            println!("getxattr error: rc = {:?}", res);
            return Err(Error::InvalidArgument);
        }
    }
    fs.set_xattr("/mp/hello.txt", "user.test1", b"hello world2")?;
    let res = fs.list_xattr("/mp/hello.txt").unwrap();
    res.iter().for_each(|x| {
        println!("list_xattr: {:?}", core::str::from_utf8(&x));
    });

    // let res = fs.remove_xattr("/mp/hello.txt", "user.test");
    // match res {
    //     Ok(_) => {}
    //     _ => {
    //         println!("remove_xattr error: rc = {:?}", res);
    //         return;
    //     }
    // }
    // println!("remove_xattr: user.test");

    println!("test_xattr Pass");
    Ok(())
}

fn test_link<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_link:");
    fs.hard_link("/mp/hello.txt", "/mp/hello1.txt")?;
    println!("hard_link: /mp/hello.txt -> /mp/hello1.txt");
    fs.soft_link("/mp/hello.txt", "/mp/hello2.txt")?;
    println!("soft_link: /mp/hello.txt -> /mp/hello2.txt");
    test_dir_ls(fs, "/mp/")?;
    println!("test_link Pass");
    Ok(())
}

fn test_rename<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_rename:");
    fs.rename("/mp/hello.txt", "/mp/hello-rename.txt")?;
    println!("rename: /mp/hello.txt -> /mp/hello-rename.txt");
    test_dir_ls(fs, "/mp/")?;
    println!("test_rename Pass");
    Ok(())
}

fn test_link_rename<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_link_rename:");
    let mut file = fs.file_builder().read(true).open("/mp/hello-rename.txt")?;
    let meta = file.metadata()?;
    assert_eq!(meta.len(), b"hello world".len() as u64);
    let mut buf = vec![0u8; meta.len() as usize];
    let res = file.read(&mut buf)?;
    assert_eq!(res, meta.len() as usize);
    assert_eq!(buf, b"hello world");
    println!("test_link_rename Pass");

    let link = fs.read_link("/mp/hello2.txt")?;
    assert_eq!(link, "/mp/hello.txt");
    Ok(())
}

fn test_mknod<T: BlockDeviceInterface>(fs: &mut FileSystem<T>) -> Result<()> {
    println!("test_mknod:");
    fs.mknod("/mp/fifo", FileType::from_char('c'), 0xfffff)?;
    fs.mknod("/mp/blk", FileType::from_char('b'), 0xff)?;
    let meta = fs.metadata("/mp/fifo")?;
    assert_eq!(meta.len(), 0);
    assert_eq!(meta.rdev(), 0xfffff);
    let meta = fs.metadata("/mp/blk")?;
    assert_eq!(meta.len(), 0);
    assert_eq!(meta.rdev(), 0xff);
    println!("test_mknod Pass");
    Ok(())
}
