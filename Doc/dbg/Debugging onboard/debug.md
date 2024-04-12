# 关于上板过程中产生的一系列问题

## Q1-启动方式发生变化

从去年的2k500到今年的2k1000，启动方式从NAND换成了MEMBLK，故而重写了去年的启动文件，删去ls_nand文件夹及引用

## Q2-Linux下链接usb_debug接口以及tftp服务器创建

根据经验，使用minicom与tftpd-hpa作为启动工具，在配置tftpd时产生如下新的经验：

- 对于配网，由于uboot下端口写死，仅可以使用:69，**需保证uboot与tftp服务器（网口或网桥）处于同一网段**
- 对于tftp方式与直接烧录内存，更推荐使用tftp，直接烧录内存经测试速率大约0.9KiB/s
- 不建议使用usb-img启动

## Q3-启动后无法正确引导

以下为Debug过程

### Sector1-做出启动调用流程图

经过对文件的检查

1. 在启动rust_main之前，进行entry.asm，对t0寄存器写零并跳转至rust_main
2. 在rust_main之中进行`bootstrap_init()`与`mem_clear()`
3. 启动检查CPUcfg，UART寄存器等

### Sector2-检查内存载入情况

首先检查编译后的.asm文件[asm_all.txt](Doc\dbg\Debugging onboard\asm_all.txt)，并在uboot使用`md 0x90000000`检查入口处是否正确

可以发现入口处确实载入64B的uImage信息头部与0x40Offset的entry.asm

之后向main函数中插入打印日志语句`println!("get in main");`，发现启动后并未进入main函数，进而对entry.asm与2k1000.rs与config.rs产生怀疑

### Sector3-检查地址

对Makefile以及config.rs进行修改，主要修改以下几个变量
- `PAGE_SIZE`
- `MEMORY_START`
- `DISK_IMAGE_BASE`
- `BLOCK_SZ`
- `UART_BASE`

并未收获改观，转而怀疑entry.asm

### Sector4-entry.asm

对照参考手册，发现entry.asm主要进行如下几步操作

- 对t0进行清零
- 对齐PC计数器
- 跳转至rust_main

通过如下汇编得知
```jirl        $t0,    $t0,    0x10 ```
在本语句中，jirl的作用为将t0寄存器向之后转移地址`bl          rust_main`移动，并重置PC计数器
可经过计算，0x10的Offset并不能跳出，故而增加为0x28,并删去冗余清零操作
```
pcaddi      $t0,    0x0
srli.d      $t0,    $t0,    0x30
slli.d      $t0,    $t0,    0x30
```
改为```sub.d       $t0,    $t0,    $t0```