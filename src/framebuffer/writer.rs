use super::pixel::PixelFormat;
use core::{fmt, ptr};

use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};
use spin::Mutex;

const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size24;
const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
const BACKUP_CHAR: char = '�';
const FONT_WEIGHT: FontWeight = FontWeight::Regular;

const LINE_SPACING: usize = 0;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 0;

const LINE_HEIGHT: usize = LINE_SPACING + CHAR_RASTER_HEIGHT.val();
const FONT_WIDTH: usize = CHAR_RASTER_WIDTH + LETTER_SPACING;

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

pub static FRAME_BUFFER: Mutex<FramebufferWriter> = Mutex::new(FramebufferWriter {
    buffer: None,
    height: 0,
    width: 0,
    stride: 0,
    pixel_format: PixelFormat::RGBA8888,
    x_pos: BORDER_PADDING,
    y_pos: BORDER_PADDING,
});

pub struct FramebufferWriter {
    buffer: Option<&'static mut [u8]>,
    width: usize,
    height: usize,
    stride: usize,
    pixel_format: PixelFormat,
    x_pos: usize,
    y_pos: usize,
}

impl FramebufferWriter {
    pub fn init(&mut self, address: u64, width: u32, height: u32, bpp: u8, stride: u32) {
        let bytes_per_pixel = bpp as usize / 8;
        let buffer_size = (stride * height) as usize;

        self.buffer = unsafe {
            Some(core::slice::from_raw_parts_mut(
                address as *mut u8,
                buffer_size,
            ))
        };
        self.width = width as usize;
        self.height = height as usize;
        self.stride = stride as usize;

        self.pixel_format = match bytes_per_pixel {
            3 => PixelFormat::RGB888,
            2 => PixelFormat::RGB565,
            4 => PixelFormat::RGBA8888,
            1 => PixelFormat::Gray8,
            _ => panic!("Unsupported pixel format"),
        };
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.buffer.as_mut().unwrap().fill(0);
    }

    fn newline(&mut self) {
        self.y_pos += LINE_HEIGHT;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    fn scroll(&mut self) {
        let scroll_height = LINE_HEIGHT;
        let src_start = scroll_height * self.stride;
        let bytes_to_copy = (self.height - scroll_height) * self.stride;

        unsafe {
            ptr::copy(
                self.buffer.as_mut().unwrap()[src_start..].as_ptr(),
                self.buffer.as_mut().unwrap().as_mut_ptr(),
                bytes_to_copy,
            );
        }

        let clear_start = (self.height - scroll_height) * self.stride;
        self.buffer.as_mut().unwrap()[clear_start..].fill(0);

        self.y_pos = self.y_pos.saturating_sub(scroll_height);
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + FONT_WIDTH;
                if new_xpos >= self.width {
                    self.newline();
                }

                let new_ypos = self.y_pos + LINE_HEIGHT;
                if new_ypos > self.height {
                    self.scroll()
                }

                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_rendered_char(&mut self, rc: RasterizedChar) {
        let x_start = self.x_pos;
        let y_start = self.y_pos;

        // 预计算边界检查
        if x_start + CHAR_RASTER_WIDTH > self.width
            || y_start + CHAR_RASTER_HEIGHT.val() > self.height
        {
            return;
        }

        let buffer = &mut self.buffer;
        let stride = self.stride;
        let format = self.pixel_format;

        rc.raster().iter().enumerate().for_each(|(y, row)| {
            let y_pos = y_start + y;
            row.iter().enumerate().for_each(|(x, &intensity)| {
                let x_pos = x_start + x;
                let index = y_pos * stride + x_pos * format.bytes_per_pixel();

                format.write_pixel(&mut buffer.as_mut().unwrap()[index..], intensity);
            });
        });

        self.x_pos += FONT_WIDTH;
    }
}

unsafe impl Send for FramebufferWriter {}
unsafe impl Sync for FramebufferWriter {}

impl fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
