/** @addtogroup NPUcore-IMPACT!!!
 * @{
 */
/**
 * @file  ext4_user.h
 * @brief Ext4 memory allocate without libc(LoongArch).
 */

#ifndef EXT4_USER_H_
#define EXT4_USER_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <ext4_types.h>

#include <stdbool.h>
#include <stdint.h>

// FOLLOW origin libc malloc
// void* ext4_user_malloc(size_t size_t);

// FOLLOW origin libc free
void ext4_user_free(void *p);

// FOLLOW origin libc calloc
void *ext4_user_calloc(size_t numitems, size_t size);

// FOLLOW origin libc realloc
void *ext4_user_realloc(void *p, size_t size);

#define EXT4_USER_BLOCK_SIZE 256
uint32_t USER_HEAP_BASE = 0x0000000000000000; // 乱写的，先验证能不能编译过

typedef struct s_block *t_block;

struct s_block {
	size_t size;	      // 数据区大小
	int free;	      // 是否是空闲块
	struct s_block *next; // 指向下个块的指针
	struct s_block *prev;
	void *ptr;
	char data[1];
};

// FOLLOW libc strlen, strcmp, strncmp
size_t strlen(char *str);
size_t strcmp(char *s1, char *s2);
size_t strncmp(char *str1, char *str2, size_t n);
void qsort(void *base, size_t num, size_t size,
	   size_t (*cmp)(const void *, const void *));

#define ALIGN4_HI(val) (((val) + 3) & (~3))

#ifdef __cplusplus
}
#endif

#endif /* EXT4_BALLOC_H_ */

/**
 * @}
 */
