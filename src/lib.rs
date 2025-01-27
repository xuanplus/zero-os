#![no_std]
#![no_main]

mod framebuffer;
mod serial;

use core::panic::PanicInfo;
use framebuffer::init_framebuffer;
use multiboot2::{BootInformation, BootInformationHeader};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main(mbi_ptr: u32, mb_magic: u32) -> ! {
    init_kernel(mbi_ptr, mb_magic);

    for i in 0..600 {
        print!("Hello world! -- {i}");
    }

    loop {}
}

fn init_kernel(mbi_ptr: u32, mb_magic: u32) {
    if mb_magic == multiboot2::MAGIC {
        let boot_info =
            unsafe { BootInformation::load(mbi_ptr as *const BootInformationHeader).unwrap() };
        let framebuffer_tag = boot_info.framebuffer_tag().unwrap().unwrap();

        init_framebuffer(
            framebuffer_tag.address(),
            framebuffer_tag.width(),
            framebuffer_tag.height(),
            framebuffer_tag.bpp(),
            framebuffer_tag.pitch(),
        );
    } else {
        panic!("Kernel start failed.");
    }
}
