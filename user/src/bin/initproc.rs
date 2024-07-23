#![no_std]
#![no_main]
// use std::net::Shutdown;

// use std::println;

use user_lib::{exec, exit, fork, shutdown, wait, waitpid, yield_};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
}

#[no_mangle]
fn main() -> i32 {
    let path = "/bin/bash\0";
    let environ = [
        "SHELL=/bash\0".as_ptr(),
        "PWD=/\0".as_ptr(),
        "LOGNAME=root\0".as_ptr(),
        "MOTD_SHOWN=pam\0".as_ptr(),
        "HOME=/root\0".as_ptr(),
        "LANG=C.UTF-8\0".as_ptr(),
        "TERM=vt220\0".as_ptr(),
        "USER=root\0".as_ptr(),
        "SHLVL=0\0".as_ptr(),
        "OLDPWD=/root\0".as_ptr(),
        "PS1=\x1b[1m\x1b[32mNPUcore-IMPACT\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0".as_ptr(),
        "_=/bin/bash\0".as_ptr(),
        "PATH=/:/bin\0".as_ptr(),
        "LD_LIBRARY_PATH=/\0".as_ptr(),
        core::ptr::null(),
    ];
    if fork() == 0 {
        exec(
            path,
            &[path.as_ptr() as *const u8, core::ptr::null()],
            &environ,
        );
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            // ECHLD is -10
            if pid == -10 {
                yield_();
                continue;
            }
            user_lib::println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid,
                exit_code,
            );
        }
    }
    //     let schedule_text: &str = "
    // \0
    // ./fantastic_text\0
    // ";
    //         let mut exit_code: i32 = 0;
    //         for line in schedule_text.lines() {
    //             let argv = [
    //                 path.as_ptr(),
    //                 "-c\0".as_ptr(),
    //                 line.as_ptr(),
    //                 core::ptr::null(),
    //             ];
    //             let pid = fork();
    //             if pid == 0 {
    //                 exec(path, &argv, &environ);
    //             } else {
    //                 waitpid(pid as usize, &mut exit_code);
    //             }
    //         }

    //     shutdown();
    0
}
