#!/bin/sh
DISK=/mnt/tmp/disk
#UDISK="-drive if=none,file=/mnt/tmp/disk,id=udisk -device usb-storage,drive=udisk,bus=usb-bus.0"
stty -isig
#QEMU=/loongson/ls3a5000/qemu/.build-la/loongarch64-softmmu/qemu-system-loongarch64
QEMU=/tmp/qemu/bin/qemu-system-loongarch64

loongson7a()
{
#BIOS=/loongson/ls3a5000/qemu-bin/qemu/share/qemu/loongarch_bios.bin
BIOS=~/usr/usr/share/qemu/loongarch_bios.bin
BIOS=/loongson/ls3a5000/uefi-loongarch/Build/LoongarchVirt/RELEASE_GCC83/FV/LOONGSONVIRT.fd
BIOS=/loongson/ls3a5000/uefi-loongarch/Build/LoongarchVirt/DEBUG_GCC83/FV/LOONGSONVIRT.fd
BIOS1=/loongson/ls3a5000/uefi-loongarch/Build/LoongarchVirt/DEBUG_GCC83/FV/LOONGSONVIRT_VARS.fd 
BIOS=/tmp/qemu/share/qemu/loongarch_bios.bin
$QEMU -M loongson7a  -drive if=pflash,file=$BIOS -serial stdio -serial vc -serial vc -m 4096 -s -device pci-ohci -device usb-kbd -device usb-tablet -net nic,model=virtio-net-pci -net user,net=10.2.0.0/24,tftp=/srv/tftp  -drive if=virtio,file=$DISK -device virtio-vga,id=video0,max_outputs=1,bus=pcie.0  -D /tmp/qemu.log "$@"  2>&1 |tee qemu.log #-drive if=virtio,id=udisk,file=usb.img
}

ls7a()
{
BIOS=/tmp/qemu/share/qemu/LS3A50007A.fd
$QEMU -M ls3a7a  -serial stdio -serial vc -drive if=pflash,file=$BIOS -drive if=pflash,file=/mnt/tmp/flash -m 4096 -device usb-kbd,bus=usb-bus.0 -device usb-tablet,bus=usb-bus.0  -hda $DISK -net nic -net user,net=10.0.2.0/24,tftp=/srv/tftp -D /tmp/qemu.log -s "$@" 2>&1 |tee qemu.log
}

ls2k500()
{
BIOS=/tmp/qemu/share/qemu/ls2k500_bios.bin
SERIAL=2 $QEMU -M ls2k500  -serial stdio -serial vc -bios $BIOS  -m 2048 -device usb-kbd,bus=usb-bus.0 -device usb-tablet,bus=usb-bus.0  -device usb-storage,drive=udisk -drive if=none,id=udisk,file=$DISK -net nic -net user,net=10.0.2.0/24,tftp=/srv/tftp -D /tmp/qemu.log -s "$@" 2>&1 #-hda $DISK
}

ls2k()
{
BIOS=/tmp/qemu/share/qemu/gzrom-dtb-la2k.bin
$QEMU -M ls2k  -serial stdio -serial vc -bios $BIOS  -m 2048 -device usb-kbd,bus=usb-bus.0 -device usb-tablet,bus=usb-bus.0  -device usb-storage,drive=udisk -drive if=none,id=udisk,file=$DISK -net nic -net user,net=10.0.2.0/24,tftp=/srv/tftp -D /tmp/qemu.log -s "$@" 2>&1 #-hda $DISK
}

bmc()
{
SERIAL=2 $QEMU -M ls2k500 -drive if=pflash,file=/tmp/image-mtd -serial vc -serial vc -serial vc -m 1024 -net nic -net user,net=10.0.2.0/24,tftp=/srv/tftp,hostfwd=tcp:0.0.0.0:8443-10.0.2.15:443,hostfwd=tcp:0.0.0.0:8022-10.0.2.15:22,hostfwd=tcp:0.0.0.0:5901-10.0.2.15:5900 -D /tmp/qemu.log -serial stdio -s "$@"
}

if [ $# -gt 0 -a ${1:0:1} = '-' ];then
func=${1:1}
shift
else
echo runqemu -ls7a/-loongarch7a/-ls2k500/-ls2k/-bmc ...
exit 0
fi

$func "$@"


#ls7a "$@"
#loongson7a "$@"
#ls2k500 "$@"
#ls2k "$@"

stty isig
