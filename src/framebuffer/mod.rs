mod pixel;
mod writer;

use crate::framebuffer::writer::FRAME_BUFFER;
use core::fmt;

pub fn init_framebuffer(address: u64, width: u32, height: u32, bpp: u8, pitch: u32) {
    FRAME_BUFFER.lock().init(address, width, height, bpp, pitch);
    FRAME_BUFFER.lock().clear();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    FRAME_BUFFER.lock().write_fmt(args).unwrap();
}
