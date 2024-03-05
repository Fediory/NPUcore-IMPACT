1. version `GLIBC_2.34‘ not found 报错

检查版本：
``` shell
strings /lib/x86_64-linux-gnu/libc.so.6 |grep GLIBC_
```
显示结果，最高只到GLIBC_2.30，由于使用的系统为ubuntu20.04，已经升级到了系统版本的最高版本了。

解决方案：添加一个高级版本系统的源，直接升级libc6。

编辑源：在/etc/apt/sources.list文件添加一行：
```
deb http://th.archive.ubuntu.com/ubuntu jammy main
```

运行升级：
``` shell
sudo apt update
sudo apt install libc6
```

参考：https://blog.csdn.net/huazhang_001/article/details/128828999

2. /opt/cross-my/libexec/gcc/loongarch64-unknown-linux-gnu/12.2.0/cc1: error while loading shared libraries: libisl.so.23: cannot open shared object file: No such file or directory

``` shell
sudo apt-get install libisl-dev
```

参考：https://stackoverflow.com/questions/33734143/gcc-unable-to-find-shared-library-libisl-so

3. make: ./la_fat: Command not found

因为os/la_fat文件开头为#!/usr/bin/expect -f。需要安装expect：
``` shell
sudo apt-get install expect
```
4. error while loading shared libraries: libssl.so.3: cannot open shared object file: No such file or directory
``` shell
sudo apt-get install openssl
```

5. unknown feature

注释掉：
```rust
// #![feature(btree_drain_filter)]
// #![feature(drain_filter)]
```

`rustc` version is `rustc 1.73.0-nightly (32303b219 2023-07-29)`