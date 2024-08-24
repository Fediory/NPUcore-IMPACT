<div align=center><img width="350" height="350" src="./logo/logo.png"/></div>

# NPUcore-IMPACT!!! [OSKernel2024 LA赛道#1]

西北工业大学，“全国大学生计算机系统能力大赛 - 操作系统设计赛(全国)- OS内核实现赛道龙芯LA2K1000分赛道” 一等奖（无特等奖）参赛作品。

队名来源：具有影响力的NPUcore，三个感叹号代表三个成员。

被带飞的队长：[内核设计与适配/文档] Yixu Feng (yixu-nwpu@mail.nwpu.edu.cn)

超级队员：[Debug高手] Yifei Zhang (yurzhang.oi@gmail.com), [硬件大神] Hanchen Zhang (jackwill17611136817@outlook.com), [场外支持] Huan Guo ([个人博客](https://guohuan78.github.io/))



## ✨ News

- **2024.08.20** 提交决赛答辩PPT。
- **2024.08.19** 现场赛完成，上板成功。
- **2024.07.31** 提交完整决赛报告。
- **2024.06.01** 提交完整初赛报告。
- **2024.05.12** FAT32文件系统解耦合。
- **2024.03.31** 初赛测例满分。
- **2024.03.20** 支持龙芯赛道的NPUcore适配成功。
- **2024.3.2** 正式组队。



## 📢 给看到这个仓库的人的一段话：

致后面参加OS比赛龙芯赛道的西工大或者是其它学校的同学们：如果你们看到了这个仓库，说明你们找对了位置，这是我们NPUcore-IMPACT最原始的仓库，是基于`NPUcore+LA`的plus版（我们后续又重构了新的版本，但在这个仓库是找不到的）。这个仓库的`NPUcore-FF`分支是我们初赛满分（`fat32`文件系统）的分支，而`ext4`分支则是我们在决赛的时候对`ext4`文件系统进行适配的实验分支，**但请注意`ext4`分支是无法跑通的，因为它不是我们debug的最终版**！

我们决赛最终阶段（线上赛+第二阶段）的代码决定不完全对外公布，原因是我们在决赛前的十五天内做了非常非常多的调整，比如：成功适配`ext4`，增加了很多新的`syscall`...。但是代价是，时间紧迫，我们的代码就是一大坨，我们真的不想把拉出来的低质量代码给大家看。因此我们仅把这个原始仓库公开给大家参考。下面是我推荐的参考链接：

1. 如果你想使用`rust`版本基于LA的arch `crate`，请参考：[NPUcore-LA2K1000-Arch](./os/src/arch/la64)

2. 如果你想了解我们初赛的Debug过程，请参考：[LA初赛测例修复](./Doc/LA初赛测例修复.md)

3. 如果你想知道我们在决赛第一阶段做了什么，请参考：[决赛第一阶段文档](./Doc/决赛第一阶段文档.pdf)

4. 如果你想知道我们的最终版`NPUcore-IMPACT`和别人不一样的地方，请参考：[决赛答辩](./Doc/决赛答辩.pptx)

5. 如果你想知道我们现场赛做了什么，请参考：[现场赛](./Doc/现场赛.pdf)

6. 如果你想知道我们最后是怎么答辩的，请参考：[决赛稿](./Doc/决赛稿.docx)

7. 如果你想使用并修改我们的`logo`（需要学会使用AE），请参考：[LOGO](./logo/)

8. 如果你在Debug的时候遇到了我们没提到的困难，请参考：[2024二等奖：NPUcore-重生之我是菜狗](https://gitlab.eduxiji.net/educg-group-26011-2376549/T202410699992491-3136/-/tree/live-splice-gh?ref_type=heads)、[2022一等奖：RISCV原版NPUcore](https://gitlab.eduxiji.net/2019301887/oskernel2022-npucore/-/tree/master/Doc)、[2023二等奖-NPUcore+LA](https://gitlab.eduxiji.net/educg-group-17066-1466467/202310699111039-2789)

9. 如果你想学习`NPUcore`的搭建过程，请参考：[NPUcore-Book](./Doc/NPUcore-Book.pdf)

10. 如果你想使用我们的代码作为`baseline`，我们推荐使用（我们修改的`NPUcore-重生之我是菜狗`队伍代码，包含部分`ext4`）：[NPUcore-lwext4](https://github.com/Fediory/NPUcore-lwext4)

11. 如果你想参考我们`Latex`的文档格式与模板，请参考：[NPUcore-IMPACT-doc](https://github.com/Fediory/NPUcore-IMPACT-doc)

12. 我们整理的龙芯参考文档：[百度网盘：密码1145](https://pan.baidu.com/s/1NsGT6fv7QUGebeAYfAHoOw?pwd=1145)

13. 我们的比赛测例：[testcases源码](https://github.com/oscomp/testsuits-for-oskernel/tree/final-2024-la)，[testcases二进制文件](./user/testcas)

14. 我们的`QEMU`环境：[QEMU](./util/qemu)

    

## 👨‍🏫 想对我的学弟学妹们说的参赛建议：

1. 请一定要重视上板，在`QEMU`上跑通不是真正的跑通。（`QEMU`和板子的区别主要是地址映射，出现问题请往这个方向查找）
1. 不要完全相信比赛的硬件以及他对应的文档，每块板子其实都是独一无二的。一旦出现位置bug，建议一看板子元件，二读`uboot`源码，别研究黑盒。
1. 同一份代码，在板子的不同时间、不同温度、不同姿态下会跑出来不一样的结果。
1. 希望学弟学妹可以从头写一个新的`NPUcore`，而不是用我们这个老版，我希望这个版本仅作为你们的一个参考。
1. 在学习阶段最好不要直接学习`NPUcore`，而是先做一下这个实验：[xv6-loongarch](https://github.com/Junkher/xv6-loongarch)
1. 我建议学弟学妹不要盲目用这个版本的`NPUcore-IMPACT`作为你的baseline，以及它的耦合度非常非常高，我们废了半天劲才解耦
1. 如果仍然选择我们的`FAT32`版本的`NPUcore-IMPACT`作为你们的baseline，那请参考我们的[所有文档](./Doc)，并先实现`vfs`，把`fs`和`fat32`完全解耦，再考虑增加新的文件系统（如果明年仍然是`EXT4`为主流）和系统调用。
1. 现在的`NPUcore-IMPACT`在功能性上仍有很多不足，如果明年仍然需要跑`ltp`测例，那一定要多加系统调用（据说明年要拿好名次，可能需要200个`syscall`）。
1. `NPUcore2022`主要做了`cache`上的优化，但是它也导致了很多功能上的问题，如果后面出了很多新的bug，请务必考虑这里，必要时可以抛弃曾经的亮点。
1. 对于我们现在的`NPUcore-IMPACT`，请把功能优先于性能考虑，虽然性能上仍有很多优化空间，但是功能上的不完善会导致一分都得不到。
1. 如果你们选择了龙芯赛道（如果明年还有的话），那么请做好完全找不到头绪的准备。
1. 如果你们时间比较充裕，在完善了功能的前提下，可以考虑参考[Pantheon](https://gitlab.eduxiji.net/T202410336992584/oskernel-2024-pantheon)进行性能优化，并尝试参考[Alien](https://gitlab.eduxiji.net/202310007101563/Alien/-/tree/main/)添加UI界面。
1. 如果是为了比赛，那么请在遵守规则的情况下，以拿到更高的分数为主，必要时候可能需要违背初心（但是我们极其不推荐，一定要注重提高自己的代码/文档/Debug水平）。
1. 如果你有其它问题，请联系我们的邮箱，或者直接在QQ群里单杀我们。
1. 如果你想复现我们的OS现象，请参考下方的教程。



## 基础环境配置
1. make、Cmake安装（辅助编译工具）
执行：
``` shell
sudo apt-get install make
sudo apt-get install cmake
```

2. 安装rust对LoongArch的编译链
    + 安装rustup（rust的安装器+版本管理器）
    
        ```bash
        rustup install nightly-2024-02-03
        ```
    
    + 安装Rust工具链
        由于LoongArch架构的交叉编译Rust工具链已经合并到上游， 目前不需要我们手动安装。  
        在 `Makefile` 中有自动的检测脚本， 只需要后续的make命令即可。
        
    + 安装交叉编译工具。本项目使用的为在x86_64下编译产生loongarch64的编译工具。  Loong Arch GCC 13： https://github.com/LoongsonLab/oscomp-toolchains-for-oskernel
        ```
		wget https://github.com/LoongsonLab/oscomp-toolchains-for-oskernel/releases/download/gcc-13.2.0-loongarch64/gcc-13.2.0-loongarch64-linux-gnu.tgz
        
		tar zxf gcc-13.2.0-loongarch64-linux-gnu.tgz
		
		# 在.bashrc中增加交叉编译器路径。假设当前路径为：/opt/gcc-13.2.0-loongarch64-linux-gnu
		export PATH=${PATH}:/opt/gcc-13.2.0-loongarch64-linux-gnu/bin
		
		# 如果配置正确，输入如下命令
		loongarch64-linux-gnu-gcc -v
		
		#会显示如下：
		Using built-in specs.
		COLLECT_GCC=loongarch64-linux-gnu-gcc
		COLLECT_LTO_WRAPPER=/home/airxs/local/gcc-13.2.0-loongarch64-linux-gnu/bin/../libexec/gcc/loongarch64-linux-gnu/13.2.0/lto-wrapper
		Target: loongarch64-linux-gnu
		Configured with: ../configure --prefix=/home/airxs/user/gnu/cross-tools --build=x86_64-cross-linux-gnu --host=x86_64-cross-linux-gnu --target=loongarch64-linux-gnu --with-sysroot=/home/airxs/user/gnu/cross-tools/sysroot --with-mpfr=/home/airxs/user/gnu/cross-tools --with-gmp=/home/airxs/user/gnu/cross-tools --with-mpc=/home/airxs/user/gnu/cross-tools --enable-__cxa_atexit --enable-threads=posix --with-system-zlib --enable-libstdcxx-time --enable-checking=release --enable-default-pie --enable-languages=c,c++,fortran,objc,obj-c++,lto
		Thread model: posix
		Supported LTO compression algorithms: zlib
		gcc version 13.2.0 (GCC) 
		```


3. 缺少部分库文件和编译rust代码出现错误的问题
   建议尝试`make clean`后， 删除对应文件夹的Cargo.lock， 尝试在Cargo.toml中删除版本限制再重新编译。

## 运行方式与运行效果
直接在根目录命令行`make`即可。 第一次运行推荐先执行一遍从而方便环境的安装和熟悉。

<details close>
<summary><b>正常情况下， 应当呈现出下列运行效果：</b></summary>

```bash
ram=0x1f17f00
length=852992 must be 16777216 bytes,run command:
trucate -s 16777216 file
to resize file
oobsize = 64





_ __ __ _ _ ___ ___ __ _ _ / ___ __ \
| | | | | |\ | | __ [__ | | |\ | | | __ | \ |
|___ |__| |__| | \| |__] ___] |__| | \| \ |__] |__/ /

Trying to boot from SPI


U-Boot 2022.04 (Jan 26 2024 - 15:42:00 +0800)

CPU: LA264
Speed: Cpu @ 900 MHz/ Mem @ 400 MHz/ Bus @ 125 MHz
Model: loongson-2k1000
Board: LS2K1000-DP
DRAM: 1 GiB
Core: 74 devices, 20 uclasses, devicetree: board
cam_disable:1, vpu_disable:1, pcie0_enable:0, pcie1_enable:1
Loading Environment from SPIFlash... SF: Detected gd25q128 with page size 256 Bytes, erase size 4 KiB, total 16 MiB
*** Warning - bad CRC, using default environment

Cannot get ddc bus
In: serial
Out: serial
Err: serial vidconsole

eth0: using random MAC address - f2:ef:a7:28:76:cd

eth1: using random MAC address - 82:98:7e:f2:f8:e4
Net: Could not get PHY for mdio@0: addr 0
Could not get PHY for mdio@1: addr 0
3No ethernet found.

************************** Notice **************************
Press c to enter u-boot console, m to enter boot menu

************************************************************
Bus otg@40000000: dwc2_usb otg@40000000: Core Release: 0.000
dwc2_usb otg@40000000: SNPSID invalid (not DWC2 OTG device): 00000000
Port not available.
Bus ehci@40060000: USB EHCI 1.00
Bus ohci@40070000: USB OHCI 1.0
scanning bus ehci@40060000 for devices... 3 USB Device(s) found
scanning bus ohci@40070000 for devices... 1 USB Device(s) found
init ls_trigger_boot and set it default value
init ls_trigger_u_kernel and set it default value
init ls_trigger_u_rootfs and set it default value
init ls_trigger_u_uboot and set it default value
Saving Environment to SPIFlash... Erasing SPI flash...Writing to SPI flash...done
OK
Autoboot in 0 seconds
SF: Detected gd25q128 with page size 256 Bytes, erase size 4 KiB, total 16 MiB
device 0 offset 0xf0000, size 0x10000
SF: 65536 bytes @ 0xf0000 Read: OK

Reset SCSI
scanning bus for devices...
Target spinup took 0 ms.
Target spinup took 0 ms.
Target spinup took 0 ms.
SATA link 3 timeout.
SATA link 4 timeout.
SATA link 5 timeout.
AHCI 0001.0000 32 slots 6 ports 1.5 Gbps 0x3f impl SATA mode
flags: 64bit ncq only
Device 0: (0:0) Vendor: ATA Prod.: QEMU HARDDISK Rev: 2.5+
Type: Hard Disk
Capacity: 100.0 MB = 0.0 GB (204800 x 512)
Device 1: (1:0) Vendor: ATA Prod.: QEMU HARDDISK Rev: 2.5+
Type: Hard Disk
Capacity: 1024.0 MB = 1.0 GB (2097152 x 512)
Device 2: (2:0) Vendor: ATA Prod.: QEMU HARDDISK Rev: 2.5+
Type: Hard Disk
Capacity: 1024.0 MB = 1.0 GB (2097152 x 512)
** No partition table - scsi 0 **
Couldn't find partition scsi 0:1
Can't set block device
Wrong Image Format for bootm command
ERROR: can't get kernel image!
Bootcmd="setenv bootargs ${bootargs} root=/dev/sda${syspart} mtdparts=${mtdparts} video=${video}; sf probe;sf read ${fdt_addr} dtb;scsi reset;ext4load scsi 0:${syspart} ${loadaddr} /boot/uImage;bootm "
Boot Kernel failed. Kernel not found or bad.
=>
=>
=>
=> fatload scsi 0 ${loadaddr} /kernel.bin;go ${loadaddr};
47739944 bytes read in 761 ms (59.8 MiB/s)
## Starting application at 0x9000000090000000 ...
[kernel] NPUcore-IMAPCT!!! ENTER!
[kernel] UART address: 0x1fe20000
[bootstrap_init] PRCfg1 { SAVE reg. number: 8, Timer bits: 48, max vector entry spacing: 7 }
[kernel] Console initialized.
last 37479 Physical Frames.
.text [0x90000000, 0x90069000)
.rodata [0x90069000, 0x90075000)
.data [0x90081000, 0x92d88000)
.bss [0x92d88000, 0x96d99000)
mapping .text section
mapping .rodata section
mapping .data section
mapping .bss section
mapping physical memory
mapping memory-mapped registers
[get_timer_freq_first_time] clk freq: 100000000(from CPUCFG)
[CPUCFG 0x0] 1351680
[CPUCFG 0x1] 66253566
[CPUCFG 0x2] 6341127
[CPUCFG 0x3] 3327
[CPUCFG 0x4] 100000000
[CPUCFG 0x5] 65537
[CPUCFG 0x6] 0
[CPUCFG 0x10] 11325
[CPUCFG 0x11] 0
[CPUCFG 0x12] 0
[CPUCFG 0x13] 0
[CPUCFG 0x14] 0
Misc { 32-bit addr plv(1,2,3):: false,false,false, rdtime allowed for plv(1,2,3):: false,false,false, Disable dirty bit check for plv(0,1,2):: false,false,false, Misalignment check for plv(0,1,2,4):: false,false,false,false }
RVACfg { rbits: 0 }
[machine_init] MMAP_BASE: 0xffffff8000000000
[kernel] Hello, world!
Testing execve :
========== START test_execve ==========
I am test_echo.
execve success.
========== END main ==========
Testing brk :
========== START test_brk ==========
Before alloc,heap pos: 12288
After alloc,heap pos: 12352
Alloc again,heap pos: 12416
========== END test_brk ==========
Testing chdir :
========== START test_chdir ==========
chdir ret: 0
current working dir : /test_chdir
========== END test_chdir ==========
Testing clone :
========== START test_clone ==========
Child says successfully!
clone process successfully.
pid:3
========== END test_clone ==========
Testing close :
========== START test_close ==========
close 3 success.
========== END test_close ==========
Testing dup2 :
========== START test_dup2 ==========
from fd 100
========== END test_dup2 ==========
Testing dup :
========== START test_dup ==========
new fd is 3.
========== END test_dup ==========
Testing exit :
========== START test_exit ==========
exit OK.
========== END test_exit ==========
Testing fork :
========== START test_fork ==========
child process.
parent process. wstatus:0
========== END test_fork ==========
Testing fstat :
========== START test_fstat ==========
fstat ret: 0
fstat: dev: 2048, inode: 5784, mode: 33279, nlink: 1, size: 52, atime: 0, mtime: 0, ctime: 0
========== END test_fstat ==========
Testing getcwd :
========== START test_getcwd ==========
getcwd: / successfully!
========== END test_getcwd ==========
Testing getdents :
========== START test_getdents ==========
open fd:3
getdents fd:456
getdents success.
lib

========== END test_getdents ==========
Testing getpid :
========== START test_getpid ==========
getpid success.
pid = 2
========== END test_getpid ==========
Testing getppid :
========== START test_getppid ==========
getppid success. ppid : 1
========== END test_getppid ==========
Testing gettimeofday :
========== START test_gettimeofday ==========
gettimeofday success.
start:12098, end:12163
interval: 65
========== END test_gettimeofday ==========
Testing mkdir_ :
========== START test_mkdir ==========
mkdir ret: -17
mkdir success.
========== END test_mkdir ==========
Testing mmap :
========== START test_mmap ==========
file len: 27
mmap content: Hello, mmap successfully!
========== END test_mmap ==========
Testing mount :
========== START test_mount ==========
Mounting dev:/dev/vdb to ./mnt
mount return: 0
mount successfully
umount return: 0
========== END test_mount ==========
Testing munmap :
========== START test_munmap ==========
file len: 27
munmap return: 0
munmap successfully!
========== END test_munmap ==========
Testing open :
========== START test_open ==========
Hi, this is a text file.
syscalls testing success!

========== END test_open ==========
Testing openat :
========== START test_openat ==========
open dir fd: 3
openat fd: 4
openat success.
========== END test_openat ==========
Testing pipe :
========== START test_pipe ==========
cpid: 3
cpid: 0
Write to pipe successfully.

========== END test_pipe ==========
Testing read :
========== START test_read ==========
Hi, this is a text file.
syscalls testing success!

========== END test_read ==========
Testing sleep :
========== START test_sleep ==========
sleep success.
========== END test_sleep ==========
Testing times :
========== START test_times ==========
mytimes success
{tms_utime:274200, tms_stime:0, tms_cutime:0, tms_cstime:0}
========== END test_times ==========
Testing umount :
========== START test_umount ==========
Mounting dev:/dev/vda2 to ./mnt
mount return: 0
umount success.
return: 0
========== END test_umount ==========
Testing uname :
========== START test_uname ==========
Uname: Linux debian 5.10.0-7-riscv64 #1 SMP Debian 5.10.40-1 (2021-05-28) riscv64
========== END test_uname ==========
Testing unlink :
========== START test_unlink ==========
unlink success!
========== END test_unlink ==========
Testing wait :
========== START test_wait ==========
This is child process
wait child success.
wstatus: 0
========== END test_wait ==========
Testing waitpid :
========== START test_waitpid ==========
This is child process
waitpid successfully.
wstatus: 3
========== END test_waitpid ==========
Testing write :
========== START test_write ==========
Hello operating system contest.
========== END test_write ==========
Testing yield :
========== START test_yield ==========
I am child process: 3. iteration 0.
I am child process: 4. iteration 1.
I am child process: 3. iteration 0.
I am child process: 4. iteration 1.
I am child process: 5. iteration 2.
I am child process: 3. iteration 0.
I am child process: 4. iteration 1.
I am child process: 5. iteration 2.
I am child process: 3. iteration 0.
I am child process: 4. iteration 1.
I am child process: 5. iteration 2.
I am child process: 3. iteration 0.
I am child process: 4. iteration 1.
I am child process: 5. iteration 2.
I am child process: 5. iteration 2.
========== END test_yield ==========
[initproc] test finish
```
</details>

在打印了大量的测试结果后退出执行。

## Makefile可用选项相关解释

### 内核编译与运行

注意，在命令后加入LOG=trace可以开启trace及以上的所有log， log从低到高等级分为trace, debug, info, warning, error  
`make run`: 编译系统，且执行虚拟机测试  
`make gdb`: 执行开启debug模式(需要配合gdb使用)， 启动虚拟机但不运行  

### 其他
`make clean`: 清理已经编译的项目（包括用户程序， 系统和FAT镜像）

