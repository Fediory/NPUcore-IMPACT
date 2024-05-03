# lwext4-rs

A crate for interfacing with [lwext4](https://github.com/gkostka/lwext4) from Rust.

## Details
You can find the details of the interface in [interface.md](interface.md).

## Usage

```rust
fn main(){
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
}
```

## Examples
```
RUST_LOG=info cargo run --example usage/tests/mkfs
```

## no_std
This crate is `no_std` compatible. You can disable the default features to use it in a `no_std` environment.

```toml
[dependencies]
lwext4-rs = { version = "0.1.0", default-features = false }
```

In the lwext4 configuration, debug output is enabled, so it relies on `printf/fflush/stdout` for output. In addition, it also relies on several functions:

1. `malloc` / `free` / `calloc` / `realloc`

2. `strcmp` / `strcpy` / `strncmp` 

3. `qsort`

To handle these dependencies, you can define these functions manually or rely on some existing implementation.

The[ tinyrlibc](https://github.com/rust-embedded-community/tinyrlibc)  library provides implementations of 1 and 2.  In order to implement `printf`, you can refer to [prinf_compat](https://docs.rs/printf-compat/0.1.1/printf_compat/). [c-ward](https://github.com/sunfishcode/c-ward) provides the implementation of `qsort`, you can copy it directly from here. In the end, all we need to implement are `fflush `and `stdout`. Usually, we only need to implement these two as empty functions.

```rust
#[no_mangle]
static stdout: usize = 0;

#[no_mangle]
extern "C" fn fflush(file: *mut c_void) -> c_int{
    assert!(file.is_null());
    0
}
```

## mkfs

```rust
cargo run -p lwext4-mkfs -- --help
```

## Reference

[lwext4 (C)](https://github.com/gkostka/lwext4)

[lwext4 (rust)](https://github.com/djdisodo/lwext4)