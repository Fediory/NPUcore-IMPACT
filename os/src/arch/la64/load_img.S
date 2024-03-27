    .section .data
    .global simg
    .global eimg
    .align 12
simg:
    .incbin "../easy-fs-fuse/rootfs-ubifs-ze.img"
eimg:
    .align 12