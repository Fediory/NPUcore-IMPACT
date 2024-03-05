# LAQEMU调试

## 环境准备

### LA GDB安装

参考：https://blog.csdn.net/greenmoss/article/details/127800221

gdb 12.1版本已经支持LoongArch架构

1. 下载安装包
```bash
wget http://ftp.gnu.org/gnu/gdb/gdb-12.1.tar.gz
```
若下载过慢，可以在windows系统下直接访问ftp链接通过浏览器下载，拷贝到Ubuntu中。

2. 解压安装包
```bash
tar -zxvf gdb-12.1.tar.gz
```

3. 预编译
```bash
cd gdb-12.1    # 进入文件夹gdb-12.1
mkdir build    # 建立文件夹build
cd build 		# 进入文件夹build
../configure --prefix=/usr --target=loongarch64-unknown-linux-gnu
```

4. 编译
```bash
make
make install
```
上述两句均需执行，可能需要较长时间，请耐心等待。

在`make install`时若出现权限不足的报错，尝试使用`sudo make install`。

成功之后：
```
See any operating system documentation about shared libraries for
more information, such as the ld(1) and ld.so(8) manual pages.
----------------------------------------------------------------------
 /usr/bin/mkdir -p '/usr/include'
 /usr/bin/install -c -m 644 ../../libctf/../include/ctf.h ../../libctf/../include/ctf-api.h '/usr/include'
 /usr/bin/mkdir -p '/usr/share/info'
 /usr/bin/install -c -m 644 ../../libctf/doc/ctf-spec.info '/usr/share/info'
make[3]: 离开目录“/home/loongson/gdb-12.1/build/libctf”
make[2]: 离开目录“/home/loongson/gdb-12.1/build/libctf”
make[1]: 对“install-target”无需做任何事。
make[1]: 离开目录“/home/loongson/gdb-12.1/build”
```

5. 运行
```bash
cd /gdb
./gdb
```
检查升级后的版本：
```
GNU gdb (GDB) 12.1
Copyright (C) 2022 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
Type "show copying" and "show warranty" for details.
This GDB was configured as "loongarch64-unknown-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<https://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
    <http://www.gnu.org/software/gdb/documentation/>.

For help, type "help".
Type "apropos word" to search for commands related to "word".
```
注意检查，第一行GDB版本为12.1，第七行GDB的config为`loongarch64-unknown-linux-gnu`。

### VScode调试配置

安装插件 C/C++ ，在工作区下面的 .vscode 目录下新建 launch.json 文件，并写入如下内容：

```json
{
   "version": "0.2.0",
   "configurations": [
      {
            "type": "cppdbg",
            "request": "launch",
            "name": "Attach to gdbserver",
            "program": "${workspaceFolder}/os/target/loongarch64-unknown-linux-gnu/debug/os",
            "miDebuggerServerAddress": "localhost:1234",
            "miDebuggerPath": "path/to/gdb-12.1/build/gdb/gdb",
            "cwd": "${workspaceRoot}/os",
      }
   ]
}
```

注意修改`miDebuggerPath`中的`path/to`为自己电脑上的路径。

## 开始调试

在`os`目录下输入`make gdb`命令，回车，进入GDB连接等待。

> 如果内核代码已被修改，通过`make new-gdb`命令，会先进行`build`，再进行`GDB`连接等待。

在内核代码中打上断点，再点击VScode面板中的开始调试（快捷键为F5），之后可以看到内核继续运行，停在所打断点处。