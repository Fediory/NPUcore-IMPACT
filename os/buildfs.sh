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
"$SUDO" dd if=/dev/zero of=${U_FAT32} bs=1M count=50
echo Making fat32 image with BLK_SZ=${BLK_SZ}
"$SUDO" mkfs.vfat -F 32 ${U_FAT32} -S ${BLK_SZ}
"$SUDO" fdisk -l ${U_FAT32}

if test -e ${U_FAT32_DIR}/fs
then 
    "$SUDO" rm -r ${U_FAT32_DIR}/fs
fi

"$SUDO" mkdir ${U_FAT32_DIR}/fs

"$SUDO" mount -f ${U_FAT32} ${U_FAT32_DIR}/fs
if [ $? ]
then
    "$SUDO" umount ${U_FAT32}
fi
"$SUDO" mount ${U_FAT32} ${U_FAT32_DIR}/fs

# build root
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/lib
"$SUDO" cp ../user/lib/${ARCH}/libc.so ${U_FAT32_DIR}/fs/lib
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/etc
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/bin
"$SUDO" mkdir -p ${U_FAT32_DIR}/fs/root
"$SUDO" sh -c "echo -e "root:x:0:0:root:/root:/bash\n" > ${U_FAT32_DIR}/fs/etc/passwd"
"$SUDO" touch ${U_FAT32_DIR}/fs/root/.bash_history

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

for programname in $(ls ../user/src/bin)
do
    "$SUDO" cp -r ../user/target/${TARGET}/${MODE}/${programname%.rs} ${U_FAT32_DIR}/fs/${programname%.rs}
done

if [ ! -f ${U_FAT32_DIR}/fs/syscall ]
then    
    "$SUDO" mkdir -p ${U_FAT32_DIR}/fs/syscall
fi

try_copy ../user/user_C_program/user/build/${ARCH}  ${U_FAT32_DIR}/fs/syscall
try_copy ../user/busybox_lua_testsuites/${ARCH} ${U_FAT32_DIR}/fs/
try_copy ../user/loongarch64/${ARCH} ${U_FAT32_DIR}/fs/
try_copy ../user/disk/${ARCH} ${U_FAT32_DIR}/fs/

"$SUDO" umount ${U_FAT32_DIR}/fs
echo "DONE"
return 0
