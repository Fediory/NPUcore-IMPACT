Memory distribution will be like the picture below.

![](./memory%20distribution.png)

## User Space

+ **trampoline(1 page)**: It contains two assembly programs: *__alltraps* and *__restore*. *__alltraps* starts at the lowest address in the page and *__restore* is above *__alltraps*. When trap occurs, pc will be setted to the entry of *__alltraps*.
+ **trap context(1 page)**: It only stores trap context in the lowest address in the page.
+ **signal trampoline(1 page)**: It contains an assembly program: *__call_sigreturn*. *__call_sigreturn* starts at the lowest address in the page. When user program has finished signal processing work, pc will be setted to the entry of *__call_sigreturn*.
+ **user stack(20 pages)**
+ **heap segment(up to 20 pages)**: It's above program segments, and it's memory is dynamic. *sbrk* syscall can set its size.
+ **program segments**: ELF file will be loaded here.

## Kernel Space
+ **trampoline(1 page)**: The ONLY use of trampoline is panic.
+ **user stack n(1 page)**: When process n is in S mode, kernel will use this area as kernel stack.
+ **guard page(1 page)**: These pages are below stack pages. They aren't allocated which means if kernel use the data in these areas will trap page fault. They protect kernel from modifying another process's stack data. 
+ **kernel program**: Kernel program will be loaded here.
+ **temporary storage area(up to 512 pages)**: This area now is used for *exec* syscall, which means file will be loaded here first.