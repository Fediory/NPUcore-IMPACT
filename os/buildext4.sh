SUDO=$(if [ $(whoami) = "root" ];then echo -n "";else echo -n "sudo";fi)
U_EXT4_DIR="../easy-fs-fuse"
U_EXT4=$1
TARGET=loongarch64-unknown-linux-gnu
BLK_SZ="2048"
MODE="release"


if [ $# -ge 3 ]; then
    MODE="$3"
fi


ARCH=$(echo "${TARGET}" | cut -d- -f1| grep -o '[a-zA-Z]\+[0-9]\+')
echo
echo Current arch: ${ARCH}
echo
"$SUDO" touch ${U_EXT4}
"$SUDO" dd if=/dev/zero of=${U_EXT4} bs=1M count=200
echo Making ext4 image with BLK_SZ=${BLK_SZ}
# "$SUDO" ../util/lwext4-mkfs -i ${U_EXT4} -b ${BLK_SZ} -e 4 -v
"$SUDO" mkfs.ext4 ${U_EXT4} -b ${BLK_SZ}
"$SUDO" fdisk -l ${U_EXT4}

if test -e ${U_EXT4_DIR}/fs
then 
    "$SUDO" rm -r ${U_EXT4_DIR}/fs
fi

"$SUDO" mkdir ${U_EXT4_DIR}/fs
"$SUDO" chmod -R 777 ${U_EXT4_DIR}
"$SUDO" mount ${U_EXT4} ${U_EXT4_DIR}/fs

# build root
"$SUDO" mkdir -p ${U_EXT4_DIR}/fs/bin
"$SUDO" mkdir -p ${U_EXT4_DIR}/fs/final
"$SUDO" mkdir -p ${U_EXT4_DIR}/fs/pre

try_copy(){
    if [ -d $1 ]
    then
        echo copying $1
        for programname in $(ls -A $1)
        do
            "$SUDO" cp -fr "$1"/"$programname" $2
        done
    else
        echo "$1" "doesn""'""t exist, skipped."
    fi
}


# build customized syscalls
if [ ! -f ${U_EXT4_DIR}/fs/syscall ]
then    
    "$SUDO" mkdir -p ${U_EXT4_DIR}/fs/user_syscall
fi

for programname in $(ls ../user/src/bin)
do
    "$SUDO" cp -r ../user/target/${TARGET}/${MODE}/${programname%.rs} ${U_EXT4_DIR}/fs/user_syscall/${programname%.rs}
done

echo user_syscall copied.
try_copy ../user/testcases ${U_EXT4_DIR}/fs/

"$SUDO" umount ${U_EXT4_DIR}/fs
echo "DONE"
