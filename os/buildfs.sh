SUDO=$(if [ $(whoami) = "root" ];then echo -n "";else echo -n "sudo";fi)
U_FAT32_DIR="../easy-fs-fuse"
U_FAT32=$1
BLK_SZ="512"
TARGET=riscv64gc-unknown-none-elf
MODE="release"
if [ $# -ge 2 ]; then
    if [ "$2"="2k1000" -o "$2"="laqemu" ]
    then
        TARGET=loongarch64-unknown-linux-gnu
        BLK_SZ="2048"
    else
        TARGET=$2
    fi
fi

if [ $# -ge 3 ]; then
    MODE="$3"
fi


ARCH=$(echo "${TARGET}" | cut -d- -f1| grep -o '[a-zA-Z]\+[0-9]\+')
echo
echo Current arch: ${ARCH}
echo
"$SUDO" touch ${U_FAT32}
"$SUDO" dd if=/dev/zero of=${U_FAT32} bs=1M count=200
echo Making fat32 image with BLK_SZ=${BLK_SZ}
"$SUDO" mkfs.vfat -F 32 ${U_FAT32} -S ${BLK_SZ}
"$SUDO" fdisk -l ${U_FAT32}

if test -e ${U_FAT32_DIR}/fs
then 
    "$SUDO" rm -r ${U_FAT32_DIR}/fs
fi

"$SUDO" mkdir ${U_FAT32_DIR}/fs
"$SUDO" chmod -R 777 ${U_FAT32_DIR}
"$SUDO" mount -f ${U_FAT32} ${U_FAT32_DIR}/fs
if [ $? ]
then
    "$SUDO" umount ${U_FAT32}
fi
"$SUDO" mount ${U_FAT32} ${U_FAT32_DIR}/fs

# build root
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/bin
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/final
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/pre

try_copy(){
    if [ -d $1 ]
    then
        echo copying $1 ';'
        for programname in $(ls -A $1)
        do
            "$SUDO" cp -fr "$1"/"$programname" $2
        done
    else
        echo "$1" "doesn""'""t exist, skipped."
    fi
}

# build customized syscalls
if [ ! -f ${U_FAT32_DIR}/fs/syscall ]
then    
    "$SUDO" mkdir -p ${U_FAT32_DIR}/fs/user_syscall
fi

for programname in $(ls ../user/src/bin)
do
    "$SUDO" cp -r ../user/target/${TARGET}/${MODE}/${programname%.rs} ${U_FAT32_DIR}/fs/user_syscall/${programname%.rs}
done

echo user_syscall copied.
try_copy ../user/testcases ${U_FAT32_DIR}/fs/

"$SUDO" umount ${U_FAT32_DIR}/fs
echo "DONE"
