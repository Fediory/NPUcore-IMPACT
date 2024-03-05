# 2k500开发板的qemu模拟器

本目录提供一个与真实硬件非常接近的龙芯2k500开发板qemu模拟器，它可以直接运行龙芯广州子公司提供的2k500开发板的固件、内核和相关软件，以便相关软件的开发和调试。

## 用法

这个qemu默认安装目录是/tmp/qemu，可以用如下命令将解压后的目录链接到/tmp/qemu:

    ln -sf `pwd`/tmp/qemu /tmp/qemu

然后cd gz; ./runqemu2k500。一切正常的话你将能看到u-boot启动，内核运行，系统启动直接出现一个shell。

启动过程中可能会看到如下内核打印：

```bash
[   17.260834] ------------[ cut here ]------------
[   17.260928] WARNING: CPU: 0 PID: 177 at drivers/tty/serial/serial_core.c:477 uart_get_baud_rate+0xfc/0x1c4
[   17.261050] Modules linked in:
[   17.261115] CPU: 0 PID: 177 Comm: (agetty) Tainted: G        W         5.10.0.lsgd-g81224fa0e223 #1
[   17.261215] Hardware name: Loongson LS2K500/LS2K500-MINI-DP, BIOS 2022.04-geba9c536 04/01/2022
[   17.261335] Stack : 00000000000003d3 900000000990f868 9000000000c8a954 900000000990c000
[   17.261463]         900000000990f7a0 0000000000000000 900000000990f7a8 9000000000ee7560
...
```

这个是由于实际硬件有多个串口，qemu没有导致的，可以用qemu命令行参数添加更多串口来避免或者忽略它。

这里发布的是qemu二进制程序，它可能依赖一些特定的软件包，我们的测试环境为ubuntu 22.04，安装上必要的依赖软件之后应该可以正常运行；如果是其他环境，可以考虑用docker跑ubuntu 22.04。

## 软件包内容

./gz: 启动qemu的命令行脚本runqemu2k500以及相应的固件和文件系统映像。
./tmp: 安装好的qemu目录

## 问题求助

遇到任何问题，可以在https://bbs.elecfans.com/group_1650求助。
