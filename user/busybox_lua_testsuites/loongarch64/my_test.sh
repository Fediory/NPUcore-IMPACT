lmbench_all bw_file_rd -P 1 512k io_only busybox
lmbench_all bw_file_rd -P 1 512k open2close busybox
lmbench_all bw_mmap_rd -P 1 512k mmap_only busybox
lmbench_all bw_mmap_rd -P 1 512k open2close busybox
