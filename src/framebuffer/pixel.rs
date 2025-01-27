#[derive(Clone, Copy)]
pub enum PixelFormat {
    RGB888,
    RGB565,
    RGBA8888,
    Gray8,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::RGB888 => 3,
            Self::RGB565 => 2,
            Self::RGBA8888 => 4,
            Self::Gray8 => 1,
        }
    }

    pub fn write_pixel(self, dest: &mut [u8], intensity: u8) {
        match self {
            Self::RGB888 => {
                dest[0] = intensity;
                dest[1] = intensity;
                dest[2] = intensity;
            }
            Self::RGB565 => {
                let val = ((intensity as u16 >> 3) << 11)
                    | ((intensity as u16 >> 2) << 5)
                    | (intensity as u16 >> 3);
                dest.copy_from_slice(&val.to_ne_bytes());
            }
            Self::RGBA8888 => {
                dest[0] = intensity;
                dest[1] = intensity;
                dest[2] = intensity;
                dest[3] = 0xFF;
            }
            Self::Gray8 => {
                dest[0] = intensity;
            }
        }
    }
}
