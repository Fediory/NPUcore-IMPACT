#include <ext4.h>
#include <ext4_fs.h>
#include <ext4_mkfs.h>
/**@brief   Mount point descriptor.*/
typedef struct ext4_mountpoint {

    /**@brief   Mount done flag.*/
    bool mounted;

    /**@brief   Mount point name (@ref ext4_mount)*/
    char name[CONFIG_EXT4_MAX_MP_NAME + 1];

    /**@brief   OS dependent lock/unlock functions.*/
    const struct ext4_lock *os_locks;

    /**@brief   Ext4 filesystem internals.*/
    struct ext4_fs fs;

    //incomplete struct definition

};