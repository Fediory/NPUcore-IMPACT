# 修改

主要修改dirtree文件中在挂载文件系统时产生的问题

存在的问题有：
- ~~mkfs不支持系统特性~~ 已解决，通过编译lwext4自带mkfs文件
- 排查流程：
    - 可以看os_logger出现的地方，
    - 目前应该是卡在块设备初始化的地方，有几个参数不对，印象为ext4_blockdev.c的bdev，具体可查看gitjournal
    - 和张先生debug的方式为在每一个报错点打印log，目前还在绝赞压栈（没找到最底层错误）

增加的模块：
- os_logger：将rs的日志库导出到c已在no_std模式下打印日志 **var_os_logger**用于打印带参数log（因为C的类型转换shit）
- mm/mod.rs：在mm中加入一个为ext4**不存在进程**但能进行分配内存的方法
- lwext4-mkfs：编译的制作镜像工具
- c库新建strlen，strcmp在no_std下可使用标准库函数

建议：
已知目前问题可能处在堆内存/块设备上面，请关注
ext4_user.h : 34
```c
#define EXT4_USER_BLOCK_SIZE 256
uint32_t USER_HEAP_BASE=0x0000000000000000; //乱写的，先验证能不能编译过
```