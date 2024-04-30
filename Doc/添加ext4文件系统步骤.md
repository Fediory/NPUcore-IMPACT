***省流版直接跳第二个一级标题***

---
---

# QA

---

## Q1：移植的时候会不会出现不适配龙芯情况

A：99%不会，目前查出来Bindgen使用Clang对C文件进行编译，之后反编译（可能不走到LLVM那一层？）回Rust，所以生成的代码最后编译时间还是走的make中的loongarch-gcc

具体编译环节的参考如下：<https://blog.csdn.net/xhhjin/article/details/81164076>

*Bindgen 应该只使用Clang部分*

---

## Q2：Alien是怎么写的

具体如图：

- Raw Lib:Lwext4-<https://github.com/gkostka/lwext4>
    - Bindgened Lib:lwext4-sys
        - Rewrited Lib:lwext4-rs

**简单来讲，就是用bindgen生成了一个链接库，再用rust调用保证干净**

---

## Q3：Lwext4写的怎么样

- C语言库，方便阅读
- 稳定性比较强，多平台测试过，支持小端序，测试过的架构有x86/AMD64，ARM系列以及其的各种嵌入式架构
    - ***自带测例***
    - *“Lwext4 code is written with endianes respect.”*
    - 自带编译库，可以作为参考
- “is an excellent choice for SD/MMC card”，还是比较符合本次比赛的
- 曾被成功移植至Risc-V（~~废话~~）

---

## Q4：我们有没有参考

有的，这是Alien中提到的Lwext4的rust转换版本，不过，写的我想紫砂

link to shit:<https://github.com/djdisodo/lwext4>

通过查他的toml可以发现：
```rust
[package]
name = "lwext4-sys"
links = "lwext4"
version = "0.1.0"
edition = "2021"
authors = ["sodo <djdisodo@gmail.com>"]
license = "GPL-2.0"
description = "ffi bind of lwext4 \"ext2/ext3/ext4 filesystem library for microcontrollers \""
repository = "https://github.com/djdisodo/lwext4"
# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]

[build-dependencies]
make-cmd = "0.1.0"
bindgen = "0.59.2"
fs_extra = "1.2.0"
```
其中fs_extra为一个文件处理管理包，详见<https://github.com/webdesus/fs_extra>
并不需要过多的其他包辅助就是了

bindgen所需自动化脚本build.rs如下：
```rust
use std::{env, fs};
use std::path::PathBuf;
use fs_extra::dir::CopyOptions;
use make_cmd::make;

fn main() {
	println!("cargo:rerun-if-changed=./lwext4");
	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
	fs_extra::dir::remove(&out_dir).unwrap();
	std::fs::create_dir_all(&out_dir).unwrap();
	let lwext4_dir = out_dir.join("lwext4");
	let mut copy_options = CopyOptions::new();
	copy_options.copy_inside = true;
	copy_options.overwrite = true;
	fs_extra::dir::copy("./lwext4", &out_dir, &copy_options).unwrap();
	make().current_dir(&lwext4_dir).arg("generic").status().unwrap();
	let build_generic_dir = lwext4_dir.join("build_generic");
	make().current_dir(&build_generic_dir).arg("lwext4").status().unwrap();
	println!("cargo:rustc-link-search={}", fs::canonicalize(build_generic_dir.join("src")).unwrap().to_str().unwrap());
	println!("cargo:rustc-link-lib=static=lwext4");
	fs_extra::dir::copy(lwext4_dir.join("include"), &build_generic_dir, &copy_options).unwrap();
	let bindings = bindgen::builder()
		.header(build_generic_dir.join("include").join("ext4.h").to_str().unwrap())
		.clang_arg(format!("-I{}", dbg!(build_generic_dir.join("include").to_str().unwrap())))
		.use_core()
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.generate().unwrap();
	bindings.write_to_file(out_dir.join("ext4.rs")).unwrap();
	let bindings = bindgen::builder()
		.header(build_generic_dir.join("include").join("ext4_inode.h").to_str().unwrap())
		.clang_arg(format!("-I{}", dbg!(build_generic_dir.join("include").to_str().unwrap())))
		.use_core()
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.generate().unwrap();
	bindings.write_to_file(out_dir.join("ext4_inode.rs")).unwrap()
}
```

还是很明显根据bindgen手册适配的，并没有过多参考价值，这也能理解为什么Alien重写了这个部分

---

## Q5：可能存在什么问题

no_std适配是最大的问题，不过Alien是可以运行的

---
---

# 总结大纲

这部分记述我们应该做什么

## I-根据lwext4或者类似的库理清楚他的函数调用，必要的话给出一个.h文件用于包装函数入口

参见bindgen手册<https://rust-lang.github.io/rust-bindgen/tutorial-2.html>

>The wrapper.h file will include all *the various headers containing declarations of structs and functions* **we would like bindings for**. In the particular case of bzip2, this is pretty easy since the entire public API is contained in a single header. ***For a project like SpiderMonkey, where the public API is split across multiple header files and grouped by functionality***, we'd want to include all those headers ***we want to bind to in this single wrapper.h entry point for bindgen***.

也就是说，对于一个比较复杂而分散的项目，我们最好给出一个包装文件.  
~~不过你直接转化最后写引用的时候也是相当折磨人就对了~~

## II-对于转换完成的rs库，视情况给出rust调用

当然，我们可以学习Alien给出新的调用接口，但是这部分是否会成为累赘以及拖慢速度仍存在异议

不过存在Sanity Check是会增加安全性就是了

## III-转换我们的fs适配新的rs库

这部分很简单，我们相当于已经拿来一个ext文件系统了，剩下的就是直接调用干活就行了。在makefile里和rust代码里加入feature就可以做到针对不同文件系统的编译与运行。

# 参考资料

Bindgen使用手册<https://rust-lang.github.io/rust-bindgen>

Alien的doc系列

<https://blog.csdn.net/yeholmes/article/details/128761222>给出了自动化脚本实例

