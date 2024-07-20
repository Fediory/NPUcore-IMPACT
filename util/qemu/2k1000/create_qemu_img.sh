#!/bin/bash

## read the document to prepare the host environment

if [ -f ./2kfs.img ] ; then
   echo "2kfs.img exist! removed"
   rm -f ./2kfs.img
fi
sudo mkdir mnt
qemu-img create -f qcow2 2kfs.img 2G

if [ $? -ne 0 ] ; then 
  echo "create image failed"
  exit -1
fi
sudo modprobe nbd maxparts=12
sudo qemu-nbd -c /dev/nbd1 ./2kfs.img

# if [ $? -ne 0 ] ; then
#   echo "connect image to nbd device failed!"
#   echo "please install nbd kernel module first!"
#   echo "   modprobe nbd maxparts=12"
#   echo "if /dev/nbd1 is already taken, change all nbd1 in this script to another one such as nbd1"
#   exit -2
# fi

sudo echo -e 'n\n\n\n\n\n\nw\nq\n'| sudo fdisk /dev/nbd1

if [ $? -ne 0 ] ; then
  echo "disk partition failed"
  exit -3
fi

sudo mkfs.vfat -F 32 /dev/nbd1p1

if [ $? -ne 0 ] ; then
  echo "mkfs.vfat -F 32 failed"
  exit -4
fi

sudo mount /dev/nbd1p1 /mnt

if [ $? -ne 0 ] ; then
  echo "mount /dev/nbd1p1 failed"
  exit -5
fi

sudo bash -c "lzcat ./rootfs-la.cpio.lzma | cpio -idmv -D /mnt &> ./cpio.log"

# if [ $? -ne 0 ] ; then
#   echo "unpack rootfs failed"
#   exit -6
# fi

sudo mkdir /mnt/boot 

sudo cp ./uImage /mnt/boot/

sudo umount /mnt

sudo qemu-nbd -d /dev/nbd1

echo "done"

