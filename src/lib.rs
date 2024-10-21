mod consts;

use crate::consts::*;
use esp_idf_hal::delay::{Ets,BLOCK};
use esp_idf_hal::i2c::*;
use esp_idf_hal::sys::EspError;

static mut DISPLAY_MODE: u8 = 0;
static mut DISPLAY_CONTROL: u8 = 0;
static mut BACKLIGHT: u8 = LCD_NOBACKLIGHT;

pub struct Lcd<'a> {
    i2c: Result<I2cDriver<'a>, EspError>,
    cols: u8,
    rows: u8,
    display_mode: u8,
    display_control: u8,
    backlight: u8,
}

impl<'a> Lcd<'a> {
    pub fn new(i2c: Result<I2cDriver<'a>, EspError>, cols: u8, rows: u8) -> Self {
        Self {
            i2c,
            cols,
            rows,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            backlight: LCD_NOBACKLIGHT,
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let display_function;
        if self.rows > 1 {
            display_function = LCD_4BITMODE | LCD_2LINE | LCD_5X8DOTS;
        } else {
            display_function = LCD_4BITMODE | LCD_1LINE | LCD_5X8DOTS;
        }
        Ets::delay_ms(50);

        self.expander_write(self.backlight)?;

        Ets::delay_ms(1000);

        for _ in 0..3 {
            self.write4bits((0x03 << 4) | self.backlight)?;
            Ets::delay_us(4500);
        }

        self.write4bits((0x02 << 4) | self.backlight)?;

        self.send(LCD_FUNCTIONSET | display_function, 0x0)?;

        self.display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.display_on()?;
        self.clear()?;

        self.display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
        self.send(LCD_ENTRYMODESET | self.display_mode, 0x0)?;

        self.send(LCD_RETURNHOME, 0x0)?;
        Ets::delay_us(2000);
        Ok(())
    }
    pub fn display_on(&mut self) -> anyhow::Result<()> {
        self.display_control |= LCD_DISPLAYON;
        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    pub fn display_off(&mut self) -> anyhow::Result<()> {
        self.display_control &= LCD_DISPLAYON;
        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    pub fn backlight_on(&mut self) {
        self.backlight = LCD_BACKLIGHT;
        self.expander_write(self.backlight)
            .expect("Failed to write to the expander while turning on the backlight");
    }

    pub fn backlight_off(&mut self) {
        self.backlight = LCD_NOBACKLIGHT;
        self.expander_write(self.backlight)
            .expect("Failed to write to the expander while turning off the backlight");
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.send(LCD_CLEARDISPLAY, 0x0)?;
        Ets::delay_us(2000);
        Ok(())
    }

    pub fn set_cursor(&mut self, col: u8, row: u8 ) -> anyhow::Result<()> {
        if row >= self.rows {
            return Err(anyhow::anyhow!("Row out of bounds"))
        }

        let row_offsets: &[u8] = match self.rows {
            1 => &[0x00],
            2 => &[0x00, 0x40],
            4 => &[0x00, 0x40, 0x14, 0x54],
            _ => return Err(anyhow::anyhow!("Invalid number of rows")),
        };

        let cmd = LCD_SETDDRAMADDR | (col + row_offsets[row as usize]);
        self.send(cmd, 0x0)?;
        Ok(())
    }

    pub fn print(&mut self, ch: char) -> anyhow::Result<()> {
        let data = ch as u8;
        self.send(data, RS)?;
        Ok(())
    }

    pub fn print_str(&mut self, str: &str ) -> anyhow::Result<()> {
        for ch in str.chars() {
            self.print(ch)?
        }
        Ok(())
    }


    fn expander_write(&mut self, data: u8) -> anyhow::Result<()> {
        let bytes = [0, data];
        self.i2c
            .as_mut()
            .unwrap()
            .write(LCD_ADDRESS, &bytes, BLOCK)?;
        Ok(())
    }

    fn pulse_enable(&mut self, data: u8) -> anyhow::Result<()> {
        let pulse = (data | EN) | self.backlight;
        self.expander_write(pulse)?;
        Ets::delay_us(1);

        let pulse = (data & !EN) | self.backlight;
        self.expander_write(pulse)?;
        Ets::delay_us(50);
        Ok(())
    }

    fn write4bits(&mut self, data: u8) -> anyhow::Result<()> {
        self.expander_write(data)?;
        self.pulse_enable(data)?;
        Ok(())
    }

    fn send(&mut self, value: u8, mode: u8) -> anyhow::Result<()> {
        let high_nibble = value & 0xf0;
        let low_nibble = (value << 4) & 0xf0;

        let high_cmd = (high_nibble | mode) | self.backlight;
        self.write4bits(high_cmd)?;

        let low_cmd = (low_nibble | mode) | self.backlight;
        self.write4bits(low_cmd)?;
        Ok(())
    }
}
