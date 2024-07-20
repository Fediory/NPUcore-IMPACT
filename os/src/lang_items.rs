use crate::arch::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.location() {
        Some(location) => {
            println!(
                "[kernel] panicked at '{}', {}:{}:{}",
                info.message(),
                location.file(),
                location.line(),
                location.column()
            );
        }
        None => println!("[kernel] panicked at '{}'", info.message()),
    }
    shutdown()
}

#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format_args!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}
