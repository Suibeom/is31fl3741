#![no_std]

use core::ops::Deref;

use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write;

use embedded_hal::prelude::_embedded_hal_blocking_i2c_WriteRead;
use rp2040_hal::pac::i2c0::RegisterBlock as I2CBlock;
use rp2040_hal::{i2c::Error, I2C};

const YS: [u8; 9] = [8, 5, 4, 3, 2, 1, 0, 7, 6];
pub struct LedMatrix {
    buffer1: [u8; 181],
    buffer2: [u8; 178],
}
impl LedMatrix {
    pub fn new() -> Self {
        LedMatrix {
            buffer1: [0; 181],
            buffer2: [0; 178],
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, r: u8, g: u8, b: u8) {
        let y = YS[y as usize];
        let offset = if x < 10 {
            ((x as u16) + 10 * (y as u16)) * 3
        } else {
            ((x as u16) + 80 + 3 * (y as u16)) * 3
        };
        if (x % 2 == 1) || (x == 12) {
            self.set_led(offset, g);
            self.set_led(offset + 1, r);
            self.set_led(offset + 2, b);
        } else {
            self.set_led(offset, b);
            self.set_led(offset + 1, g);
            self.set_led(offset + 2, r);
        }
    }

    pub fn set_led(&mut self, index: u16, value: u8) {
        if index < 180 {
            self.buffer1[(index + 1) as usize] = value;
        } else {
            self.buffer2[(index - 180 + 1) as usize] = value;
        }
    }
}

pub struct Is31<'a, I2Cn, Pins>
where
    I2Cn: Deref<Target = I2CBlock>,
{
    i2c_device: &'a mut I2C<I2Cn, Pins>,
    address: u8,
    current_page: u8,
}
impl<'a, I2Cn, Pins> Is31<'a, I2Cn, Pins>
where
    I2Cn: Deref<Target = I2CBlock>,
{
    pub fn new(i2c: &'a mut I2C<I2Cn, Pins>) -> Self {
        let mut n = Self {
            i2c_device: i2c,
            address: 0x30,
            current_page: 0,
        };
        n.reset().unwrap();

        n
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.i2c_device.write(self.address, &data)
    }

    fn write_read(&mut self, data: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.i2c_device.write_read(self.address, data, buffer)
    }

    fn reset(&mut self) -> Result<(), Error> {
        if self.current_page != 4 {
            self.set_page(4)?;
        }
        self.write(&[0x3f, 0xae])
    }

    fn unlock(&mut self) -> Result<(), Error> {
        self.write(&[0xfe, 0xc5])
    }
    fn set_page(&mut self, page: u8) -> Result<(), Error> {
        self.unlock()?;
        self.write(&[0xfd, page])
            .and_then(|()| Ok(self.current_page = page))
    }

    pub fn set_global_voltage(&mut self, voltage: u8) -> Result<(), Error> {
        if self.current_page != 4 {
            self.set_page(4)?;
        }
        self.write(&[1, voltage])
    }

    pub fn get_global_voltage(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if self.current_page != 4 {
            self.set_page(4)?;
        }
        self.write_read(&[1], buffer)
    }
    pub fn set_software_shutdown_mode_off(&mut self) -> Result<(), Error> {
        if self.current_page != 4 {
            self.set_page(4)?;
        }
        let buffer = &mut [0];
        self.write_read(&[0], buffer)?;
        let new_val = buffer[0] | 0b0000_0001;
        self.write(&[0, new_val])
    }
    pub fn get_software_shutdown_mode(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if self.current_page != 4 {
            self.set_page(4)?;
        }
        self.write_read(&[0], buffer)
    }
    pub fn set_all_led_scales(&mut self, scale: u8) -> Result<(), Error> {
        if self.current_page != 2 {
            self.set_page(2)?;
        }
        let buff = &mut [scale; 181];
        buff[0] = 0;
        self.write(buff)?;

        self.set_page(3)?;

        let buff = &mut [scale; 178];
        buff[0] = 0;
        self.write(buff)
    }
    pub fn set_led_0(&mut self) -> Result<(), Error> {
        if self.current_page != 0 {
            self.set_page(0)?;
        }
        self.write(&[0, 0xaa])?;
        self.set_page(2)?;
        self.write(&[0, 0xaa])
    }
    pub fn set_led_1(&mut self) -> Result<(), Error> {
        if self.current_page != 0 {
            self.set_page(0)?;
        }
        self.write(&[1, 0xff, 0xff, 0xff])?;
        self.set_page(2)?;
        self.write(&[1, 0xff, 0xff, 0xff])
    }
    pub fn set_leds(&mut self) -> Result<(), Error> {
        if self.current_page != 0 {
            self.set_page(0)?;
        }
        self.write(&[1, 0xff, 0xff, 0xff])?;
        self.set_page(1)?;
        self.write(&[1, 0xff, 0xff, 0xff])?;
        self.set_page(2)?;
        self.write(&[1, 0xff, 0xff, 0xff])?;
        self.set_page(3)?;
        self.write(&[1, 0xff, 0xff, 0xff])
    }

    pub fn write_led_matrix(&mut self, matrix: &LedMatrix) -> Result<(), Error> {
        if self.current_page != 0 {
            self.set_page(0)?;
        }
        self.write(&matrix.buffer1)?;
        self.set_page(1)?;
        self.write(&matrix.buffer2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
