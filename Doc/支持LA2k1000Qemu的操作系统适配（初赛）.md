# LA2k1000移植步骤

先前的NPUcore已经移植到了LA2k500的板子上，但是本次比赛的设备转换为了LA2k1000，因此我们需要对2k1000进行重适配。这里只展示从2k500到2k1000的适配过程。

## 1. Qemu2k1000启动

- 在比赛官网下载官方提供的[Qemu](https://gitlab.educg.net/wangmingjian/os-contest-2024-image/-/raw/master/qemu-2k1000-static.20240126.tar.xz)，并调整qemu相关启动参数。在初赛官网上，官方也给出了所用qemu启动完整命令。我们具体的配置详情，可以参考`./util/qemu`文件夹内查看。

```bash
qemu-system-loongarch64 -M ls2k -serial stdio -serial vc -drive if=pflash,file=/tmp/qemu/2k1000/u-boot-with-spl.bin -m 1024 -device usb-kbd,bus=usb-bus.0 -device usb-tablet,bus=usb-bus.0 \
    -device usb-storage,drive=udisk -drive if=none,id=udisk,file=/tmp/disk -net nic -net user,net=10.0.2.0/24,tftp=/srv/tftp -vnc :0 -D /tmp/qemu.log \
    -s -hda /tmp/qemu/2k1000/2kfs.img -hdb file=sdcard.img
```

- 为了适配测试平台qemu中load地址为0x9000000090000000，而之前为0x90000000000000，修改了/os/src/linker.in.ld中的BASE_ADDRESS = 0x0000000090000000;之前为全零。

- 同时根据比赛初赛要求，我们需要在文件根目录下进行`make`，因此我们在根目录中添加makefile，引导至`./os`目录。并且在`./os/makefile`中，我们重构了makefile，且加入了在根目录生成kernel.bin的相关命令。

- 2k1000和2k500的串口地址不同，因此我们在`os/src/arch/la64/board/2k1000.rs`中定义。2k1000使用memblock的方式，不使用nand。其中UART的base地址，是我们基于龙芯2k1000说明书自行计算得来，因为过程较为复杂，这里不详细说明了。下方附上我们的代码：

  ```rust
  use crate::drivers::block::MemBlockWrapper;
  pub const MMIO: &[(usize, usize)] = &[];
  
  pub type BlockDeviceImpl = MemBlockWrapper;
  
  pub const BLOCK_SZ: usize = 2048;
  pub const UART_BASE: usize = 0x1fe2_0000;
  ```

  U-Boot源码也证明了我们的计算是正确的。

  出处：U-Boot源码解析（龙芯版）章节5.3.2。指向U-Boot源码：u-boot-2022.04-2k1000
  ```asm
  /arch/loongarch/mach-loongson/ls2k100/lowlevel_init.S:394
  
  #define UART_REF_CLK	125000000
  #define UART_DIV_HI	 (((UART_REF_CLK + (115200*8)) / (115200*16)) >> 8)
  #define UART_DIV_LO	(((UART_REF_CLK + (115200*8)) / (115200*16)) & 0xff)
  ENTRY(init_serial)
  	or  a4, ra, zero      /*返回地址写入a4*/
  
  	li.d	a0, CONSOLE_BASE_ADDR
      /*UART0寄存器（映射）基址：0x8000_0000_1fe2_0000*/
      /*/include/configs/loongson_2k1000.h*/
      /*/arch/loongarch/mach-loongson/include/mach/ls2k1000/ls2k1000.h*/
  ```

- 到此为止，基于2k1000qemu的适配的部分已经结束了，接下来便是启动之后的过程。这里有一些细节和问题我们无法完全提及，需要查看我们构建相关的源码来自行补充。

## 2. 载入NPUcore内核

在前面的makefile中，我们已经在`./easy-fs-fuse`中生成了uImage和测例镜像。我们需要将uImage通过tftp传入到qemu中去，经由自带的u-boot引导启动NPUcore。为此我们写了一个expect脚本来处理。

- 首先按c来进入user界面，以使用正常的u-boot，不至于进入linux界面。接着通过`tftpboot uImage`来将uImage载入。（在启动qemu的时候，tftp的文件夹要选择`./easy-fs-fuse`）最后bootm进入NPUcore中。下面给出`./os/la_fat`的详细expect脚本。

```bash
#!/usr/bin/expect -f
set timeout -1
spawn {*}$argv
set bin_name [lindex $argv 0];

expect "Device(s) found"
send "ccccc\n"


expect "=>"
send "tftpboot uImage\n\n"


expect "Bytes transferred"
expect "=>"
send "bootm\n"


interact
```

## 3. NPUcore启动

- 首先我们将`os/src/arch/la64/entry.asm`中修改成如下部分，以适配2k1000的启动过程。修改后产生bug，需要`os/src/arch/la64/laflex.rs` 处增大dirty数组大小（在config配置中的`pub const DIRTY_WIDTH: usize = 0x100_0000;`），而后成功运行。

```asm
# 将os/src/arch/la64/entry.asm中的_start部分改为如下
_start:
    pcaddi      $t0,    0x0
    srli.d      $t0,    $t0,    0x30
    slli.d      $t0,    $t0,    0x30
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x181   # Make sure the window remains the same after the switch.
    sub.d       $t0,    $t0,    $t0
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x180
    pcaddi      $t0,    0x0
    slli.d      $t0,    $t0,    0x10
    srli.d      $t0,    $t0,    0x10
    jirl        $t0,    $t0,    0x10    # 跳0段的下一条指令
    # The barrier
    sub.d       $t0,    $t0,    $t0
    csrwr       $t0,    0x181
    sub.d       $t0,    $t0,    $t0
    la.global $sp, boot_stack_top
    bl          rust_main
```

- 因暂时无法直接从qemu上获取比赛测例（初赛阶段），所以我们选择自行下载[测例](https://github.com/LoongsonLab/oscomp-testsuits-loongarch)，并放入`./user`文件夹中编译，并直接将全部测例嵌入到内核里。这样做方便调试，但代价是kernel.bin会比较大和冗余。

  - 自行打包`rootfs-ubifs-ze.img·，在内核main函数中调用move_to_high_address函数实现内核插入img的代码。

    ```rust
    fn move_to_high_address() {
        extern "C" {
            fn simg();
            fn eimg();
        }
        unsafe {
            let img = core::slice::from_raw_parts(
                simg as usize as *mut u8,
                eimg as usize - simg as usize
            );
            // 从DISK_IMAGE_BASE到MEMORY_END
            let mem_disk = core::slice::from_raw_parts_mut(
                DISK_IMAGE_BASE as *mut u8,
                0x800_0000
            );
            mem_disk.fill(0);
            mem_disk[..img.len()].copy_from_slice(img);
        }
    }
    ```

  - 添加`load_img.S`文件，导入使得img文件编译时导入进内核。
  
    ```asm
        .section .data
        .global simg
        .global eimg
        .align 12
    simg:
        .incbin "../easy-fs-fuse/rootfs-ubifs-ze.img"
    eimg:
        .align 12
    ```
  
  - `./user/src/bin/initproc.rs`修改为内核启动自动执行测例，然后退出。我们采用的方法是，在测例中写一个`run_all.sh`脚本来依次执行所有测例，在运行时进入命令行后直接跑脚本，最后shutdown退出。
  
  - 评测平台要求运行完程序后自行退出，`os/src/arch/la64/sbi.rs处`实现shutdown函数（当前为未实现状态，参考龙芯2k1000手册电源管理部分，实现s5状态软关机。
  
    ```rust
    // os/src/arch/la64/sbi.rs
    pub fn shutdown() -> ! {
        // 电源管理模块设置为s5状态，软关机
        unsafe{
            ((0x1FE27000 + 0x14) as *mut u32).write_volatile(0b1111<<10);
        }
        loop {}
    }
    ```
  
    到这里我们的适配修改基本完成了，之后需要做的就是针对不同测例，完善kernel的debug环节。