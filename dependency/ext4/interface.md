# Interface

|                          | lwext4-c                                        | lwext4-rs                                              |
| ------------------------ | ----------------------------------------------- |--------------------------------------------------------|
| :heavy_check_mark:       | `ext4_device_register`/`ext4_device_unregister` | `RegisterHandle::register` / `drop`                    |
| :heavy_check_mark:       | `ext4_mount`/`ext4_umount`                 | `MountHandle::mount` / `drop`                  |
| :heavy_check_mark:       | `ext4_journal_start`/`ext4_journal_stop`        | `FileSystem::new` / `drop`                             |
| :heavy_check_mark:       | `ext4_recover`                                  | `MountHandle::mount`                                   |
| :heavy_check_mark:       | `ext4_mount_point_stats`                        | `MountHandle::stats`                                   |
| :heavy_check_mark:       | `ext4_cache_write_back`                         | `FileSystem::new` / `drop`                             |
| :heavy_check_mark:       | `ext4_cache_flush`                              | `File::flush`                                          |
| :heavy_check_mark:       | `ext4_fremove`                                  | `FileSystem::remove_file`                              |
| :heavy_check_mark:       | `ext4_flink`                                    | `FileSystem::hard_link`                                |
| :heavy_check_mark:       | `ext4_frename`                                  | `FileSystem::rename`                                   |
| :heavy_check_mark:       | `ext4_fopen`/`ext4_fopen2`/`ext4_fclose`        | `OpenOptions::open` / `drop`                         |
| :heavy_check_mark:       | `ext4_ftruncate`                                | `File::set_len`                                        |
| :heavy_check_mark:       | `ext4_fread`                                    | `File::read`                                           |
| :heavy_check_mark:       | `ext4_fwrite`                                   | `File::write`                                          |
| :heavy_check_mark:       | `ext4_fseek`                                    | `File::seek`                                           |
| :heavy_check_mark:       | `ext4_raw_inode_fill`                           | `File::metedata` / `FileSystem::metedata`              |
| :heavy_check_mark:       | `ext4_inode_exist`                              | `FileSystem::exists`                                   |
| :heavy_check_mark:       | `ext4_mode_set`                                 | `File::set_permissions` / `FileSystem::set_permissions` |
| :heavy_check_mark:       | `ext4_atime_set`/`ext4_mtime_set`/`ext4_ctime_set` | `File::set_times` / `File::set_modified` / `FileSystem::set_times` / `FileSystem::set_modified` |
| :heavy_check_mark:       | `ext4_fsymlink`                                 | `FileSystem::soft_link`                                |
| :heavy_check_mark:       | `ext4_readlink`                                 | `FileSystem::read_link`                                |
| :heavy_check_mark: | `ext4_mknod`                                    | `FileSystem::mknod`                                     |
| :heavy_check_mark:       | `ext4_setxattr`                                 | `FileSytem::set_xattr`                                 |
| :heavy_check_mark:       | `ext4_getxattr`                                 | `FileSystem::get_xattr`                                |
| :heavy_check_mark:       | `ext4_listxattr`                                | `FileSystem::list_xattr`                               |
| :grey_exclamation:       | `ext4_removexattr`                              | `FileSystem::remove_xattr`                             |
| :heavy_check_mark:       | `ext4_dir_rm`                                   | `FileSystem::remove_dir` / `FileSystem::remove_dir_all` |
| :heavy_check_mark:       | `ext4_dir_mv`                                   | `FileSystem::rename`                                   |
| :heavy_check_mark:       | `ext4_dir_mk`                                   | `FileSystem::create_dir` / `FileSystem::create_dir_all` |
| :heavy_check_mark:       | `ext4_dir_open`/`ext4_dir_close`                | `FileSystem::readdir`                                  |
| :heavy_check_mark:       | `ext4_dir_entry_next`                           | `ReadDir::next`                                        |
| :heavy_check_mark:       | `ext4_dir_entry_rewind`                         | `ReadDir::rewind`                                      |
| :heavy_check_mark: | `ext4_owner_set` | `FileSystem::chown` |
| :heavy_check_mark: | `ext4_ftell` | `FileSystem::stream_position` |



