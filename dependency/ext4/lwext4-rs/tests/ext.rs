use embedded_io::{Read, Seek, SeekFrom, Write};
use lwext4_rs::FsType::Ext4;
use lwext4_rs::*;
use std::fs::OpenOptions;

type FS = FileSystem<DefaultInterface<std::fs::File>>;
fn mkfs() -> FS {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("./ext_image")
        .unwrap();
    file.set_len(1024 * 1024 * 5).unwrap(); // 5GB
    let mut config = BlockDeviceConfig::default();
    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = file.metadata().unwrap().len();
    config.part_offset = 0;
    config.block_count = config.part_size / bs;

    println!("config: {:#x?}", config);
    // set_debug_mask(DebugFlags::ALL);
    let blk = DefaultInterface::new_device(file, config);
    let fs = FsBuilder::new()
        .ty(Ext4)
        .journal(true)
        .block_size(2048)
        .label("ext4fs")
        .build(blk)
        .unwrap();
    let blk = fs.take_device();
    let register_handler = RegisterHandle::register(blk, "ext4fs".to_string()).unwrap();
    let mount_handler = MountHandle::mount(register_handler, "/".to_string(), true, false).unwrap();
    let fs = FileSystem::new(mount_handler).unwrap();
    fs
}

fn rm_image() {
    std::fs::remove_file("./ext_image").unwrap();
}
#[test]
fn main_test() {
    let mut fs = mkfs();
    create_file_test(&mut fs);
    create_dir_test(&mut fs);
    simple_link_test(&mut fs);
    special_file_test(&mut fs);
    readdir_test(&mut fs);
    rename_test(&mut fs);
    attr_test(&mut fs);
    write_read_test(&mut fs);

    remove_file_test(&mut fs);
    remove_dir_test(&mut fs);
    rm_image();
}

fn create_file_test(fs: &mut FS) {
    let file_builder = fs.file_builder();
    for i in 0..50 {
        let res = file_builder
            .clone()
            .mode(0o644)
            .write(true)
            .create(true)
            .open(format!("/file{}.txt", i));
        assert!(res.is_ok(), "create file failed: {:?}", res.err());
    }
    let meta = fs.metadata("/file1.txt");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), 0);
    assert_eq!(meta.is_file(), true);
    assert_eq!(meta.permissions(), Permissions::from_mode(0o644));
}

fn create_dir_test(fs: &mut FS) {
    for i in 0..50 {
        let res = fs.create_dir(format!("/dir{}", i));
        assert!(res.is_ok(), "create dir failed: {:?}", res.err());
        let res = fs.create_dir(format!("/tmp/dir{}", i));
        assert!(res.is_ok(), "create dir failed: {:?}", res.err());
        let res = fs.create_dir_all(format!("/tmp1/dir{}", i));
        assert!(res.is_ok(), "create dir failed: {:?}", res.err());
    }
    let meta = fs.metadata("/dir1");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), 4096);
    assert_eq!(meta.is_dir(), true);
    assert_eq!(meta.permissions(), Permissions::from_mode(0o777));
    let meta = fs.metadata("/tmp/dir1");
    assert!(meta.is_ok());
    let meta = fs.metadata("/tmp1/dir1");
    assert!(meta.is_ok());
}

fn simple_link_test(fs: &mut FS) {
    let link = fs.file_builder().write(true).create(true).open("/link");
    assert!(link.is_ok());
    let link = link.unwrap();
    for i in 0..50 {
        let res = fs.hard_link("/link", format!("/link{}", i));
        assert!(res.is_ok(), "create link failed: {:?}", res.err());
    }
    let meta = link.metadata();
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), 0);
    assert_eq!(meta.is_file(), true);
    assert_eq!(meta.permissions(), Permissions::from_mode(0o666));
    assert_eq!(meta.nlink(), 51);
    let res = fs.soft_link("/link", "/symlink");
    assert!(res.is_ok(), "create symlink failed: {:?}", res.err());
    let meta = fs.metadata("/symlink");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), b"/link".len() as _);
    assert_eq!(meta.is_file(), false);
    assert_eq!(meta.is_symlink(), true);
    let str = fs.read_link("/symlink");
    assert_eq!(str, Ok("/link".to_string()));
}

fn special_file_test(fs: &mut FS) {
    let dir = fs.create_dir("/dev");
    assert!(dir.is_ok());
    for i in 0..10 {
        let f = fs.mknod(format!("/dev/c{}", i), FileType::from_char('c'), i);
        assert_eq!(f.is_ok(), true);
        let f = fs.mknod(format!("/dev/b{}", i), FileType::from_char('b'), i);
        assert_eq!(f.is_ok(), true);
        let f = fs.mknod(format!("/dev/f{}", i), FileType::from_char('s'), i);
        assert_eq!(f.is_ok(), true);
        let f = fs.mknod(format!("/dev/p{}", i), FileType::from_char('p'), i);
        assert_eq!(f.is_ok(), true);
    }
    let meta = fs.metadata("/dev/c1");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), 0);
    assert_eq!(meta.file_type().is_char_device(), true);
    assert_eq!(meta.rdev(), 1);
    // dir
    let d = fs.mknod("/dev/zero", FileType::from_char('d'), 0);
    assert_eq!(d.is_err(), true);
    // file
    let f = fs.mknod("/dev/zero", FileType::from_char('-'), 0);
    assert_eq!(f.is_err(), true);
}

fn readdir_test(fs: &mut FS) {
    let entries = fs.readdir("/dev");
    assert!(entries.is_ok());
    let entries = entries.unwrap().collect::<Vec<DirEntry>>();
    assert_eq!(entries.len(), 40 + 2, "{:?}", entries); // 40 + "." + ".."
}

fn rename_test(fs: &mut FS) {
    let dir = fs.create_dir("/rename/d1");
    let dir_f1 = fs
        .file_builder()
        .write(true)
        .create(true)
        .open("/rename/d1/f1");
    let file = fs
        .file_builder()
        .write(true)
        .create(true)
        .open("/rename/f2");
    assert!(dir.is_ok());
    assert!(file.is_ok());
    assert!(dir_f1.is_ok());

    let res = fs.rename("/rename/f2", "/rename/f3");
    assert!(res.is_ok(), "rename file failed: {:?}", res.err());
    let res = fs.rename("/rename/d1/f1", "/rename/d1/f2");
    assert!(res.is_ok(), "rename file failed: {:?}", res.err());
    let res = fs.rename("/rename/d1", "/rename/d2");
    assert!(res.is_ok(), "rename dir failed: {:?}", res.err());
    let file = fs.file_builder().read(true).open("/rename/d2/f2");
    assert!(file.is_ok());
}

fn attr_test(fs: &mut FS) {
    let res = fs.set_xattr("/link", "user.test", b"hello");
    assert!(res.is_ok(), "set_xattr failed: {:?}", res.err());
    let res = fs.set_xattr("/link", "user.test1", b"hello1");
    assert!(res.is_ok(), "set_xattr failed: {:?}", res.err());
    let res = fs.get_xattr("/link", "user.test");
    assert!(res.is_ok(), "get_xattr failed: {:?}", res.err());
    let res = res.unwrap();
    assert_eq!(res, b"hello");
    let res = fs.list_xattr("/link");
    assert!(res.is_ok(), "list_xattr failed: {:?}", res.err());
    let res = res.unwrap();
    assert!(res.contains(&b"user.test".to_vec()));
    assert!(res.contains(&b"user.test1".to_vec()));
    let res = fs.set_permissions("/link", Permissions::from_mode(0o222));
    assert!(res.is_ok(), "set_permissions failed: {:?}", res.err());
    let times = FileTimes::new()
        .set_accessed(Time::from_extra(1, None))
        .set_created(Time::from_extra(2, None))
        .set_modified(Time::from_extra(3, None));
    let res = fs.set_times("/link", times);
    assert!(res.is_ok(), "set_times failed: {:?}", res.err());
    let res = fs.chown("/link", Some(1), Some(1));
    assert!(res.is_ok(), "chown failed: {:?}", res.err());
    let meta = fs.metadata("/link");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.permissions(), Permissions::from_mode(0o222));
    assert_eq!(meta.accessed(), Time::from_extra(1, Some(0)));
    assert_eq!(meta.created(), Time::from_extra(2, Some(0)));
    assert_eq!(meta.modified(), Time::from_extra(3, Some(0)));
    assert_eq!(meta.uid(), 1);
    assert_eq!(meta.gid(), 1);
}

fn write_read_test(fs: &mut FS) {
    let file = fs.file_builder().read(true).write(true).open("/link");
    assert!(file.is_ok());
    let mut file = file.unwrap();
    const W_LEN: usize = 1024;
    const W_EPOCH: usize = 1024; // 1GB
    let mut w_buf = vec![0u8; W_LEN];
    for i in 0..W_EPOCH {
        w_buf.fill((i % 256) as u8);
        let w = file.write(&w_buf);
        assert!(w.is_ok(), "write failed: {:?}", w.err());
        let w = w.unwrap();
        assert_eq!(w, W_LEN, "write len not match, epoch: {}", i);
    }
    let mut r_buf = vec![0u8; W_LEN];
    let res = file.rewind();
    assert!(res.is_ok(), "rewind failed: {:?}", res.err());
    for i in 0..W_EPOCH {
        w_buf.fill((i % 256) as u8);
        let r = file.read(&mut r_buf);
        assert!(r.is_ok(), "read failed: {:?}", r.err());
        assert_eq!(r.unwrap(), W_LEN, "read len not match, epoch: {}", i);
        assert_eq!(w_buf, r_buf);
    }
    let r = file.read(&mut r_buf);
    assert_eq!(r, Ok(0));
    let pos = file.stream_position();
    assert_eq!(pos, Ok(W_LEN as u64 * W_EPOCH as u64));
    let res = file.seek(SeekFrom::Start(1024));
    assert_eq!(res, Ok(1024));
    let pos = file.stream_position();
    assert_eq!(pos, Ok(1024));

    let res = file.set_len(1024);
    assert!(res.is_ok(), "set_len failed: {:?}", res.err());
    let meta = fs.metadata("/link");
    assert!(meta.is_ok());
    let meta = meta.unwrap();
    assert_eq!(meta.size(), 1024);
    let res = file.flush();
    assert!(res.is_ok(), "flush failed: {:?}", res.err());
}

fn remove_file_test(fs: &mut FS) {
    let res = fs.remove_file("/link");
    assert!(res.is_ok(), "remove file failed: {:?}", res.err());
    let res = fs.remove_file("/symlink");
    assert!(res.is_ok(), "remove file failed: {:?}", res.err());
    for i in 0..50 {
        let res = fs.remove_file(format!("/file{}.txt", i));
        assert!(res.is_ok(), "remove file failed: {:?}", res.err());
    }
    for i in 0..50 {
        let res = fs.remove_file(format!("/link{}", i));
        assert!(res.is_ok(), "remove file failed: {:?}", res.err());
    }
}

fn remove_dir_test(fs: &mut FS) {
    let res = fs.remove_dir("/dev");
    assert!(res.is_ok(), "remove dir failed: {:?}", res.err());
    let res = fs.remove_dir("/rename");
    assert!(res.is_ok(), "remove dir failed: {:?}", res.err());
    for i in 0..50 {
        let res = fs.remove_dir(format!("/dir{}", i));
        assert!(res.is_ok(), "remove dir failed: {:?}", res.err());
        let res = fs.remove_dir(format!("/tmp/dir{}", i));
        assert!(res.is_ok(), "remove dir failed: {:?}", res.err());
        let res = fs.remove_dir_all(format!("/tmp1/dir{}", i));
        assert!(res.is_ok(), "remove dir failed: {:?}", res.err());
    }
}
