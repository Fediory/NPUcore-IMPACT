# LA初赛测例修复

在初赛中，我们初次提交时为85分，需要修改openat、mmap、munmap、pipe、dup2测例。其中mmap和munmap经过反汇编发现，其中调用了我们没有支持的statx syscall。因此，我们决定优先添加statx。

![statx](./assert\statx.png)

我们对照着man手册依次添加statx的相关结构体和方法，详情请参考`./os/src/syscall/fs.rs`中的sys_statx函数部分。对于这个函数，我们在初期的syscall.rs中传参错误，导致后期走了许多弯路，耽误了很多时间。目前已经修复。

- openat：为进程分配文件描述符时，应优先分配当前未被分配的最小的文件描述符。

```diff
--- a/os/src/fs/mod.rs
+++ b/os/src/fs/mod.rs
@@ -336,6 +337,8 @@ impl FdTable {
	   	match self.inner[fd].take() {
         Some(file_descriptor) => {
	             self.recycled.push(fd as u8);
+                // FIXME: shit here, replace this with balanced binary tree
+                self.recycled.sort_by(|a, b| b.cmp(a));
             Ok(file_descriptor)
        }
         None => Err(EBADF),
```

- pipe: 注释掉了如下的代码，疑似是曾经面向测例编程的部分，完全没看懂实际意思。（在之后debug中，如果还有过不了的情况会优先考虑这里）

```diff
  --- a/os/src/fs/dev/pipe.rs
  +++ b/os/src/fs/dev/pipe.rs
  @@ -269,13 +270,13 @@ impl File for Pipe {
           }
           let mut read_size = 0usize;
           loop {
-            let task = current_task().unwrap();
-            let inner = task.acquire_inner_lock();
-            if !inner.sigpending.difference(inner.sigmask).is_empty() {
-                return ERESTART as usize;
-            }
-            drop(inner);
-            drop(task);
+            // let task = current_task().unwrap();
+            // let inner = task.acquire_inner_lock();
+            // if !inner.sigpending.difference(inner.sigmask).is_empty() {
+            //     return ERESTART as usize;
+            // }
+            // drop(inner);
+            // drop(task);
               let mut ring = self.buffer.lock();
               if ring.status == RingBufferStatus::EMPTY {
                   if ring.all_write_ends_closed() {
```

- dup2：这个比较简单，因为dup2的行为完全正确，只是我们之前的fd限制在64，而测例设置的fd为100。因此我们将限制提升到128后则通过。
