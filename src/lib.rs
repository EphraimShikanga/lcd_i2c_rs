mod consts;

use crate::consts::*;
use esp_idf_hal::delay::{FreeRtos, Ets,BLOCK};
use esp_idf_hal::i2c::*;
use esp_idf_hal::sys::EspError;

static mut DISPLAY_MODE: u8 = 0;
static mut DISPLAY_CONTROL: u8 = 0;
static mut BACKLIGHT: u8 = LCD_NOBACKLIGHT;

pub struct Lcd<'a> {
    i2c: Result<I2cDriver<'a>, EspError>,
    display_mode: u8,
    display_control: u8,
    backlight: u8,
}

impl<'a> Lcd<'a> {
    pub fn new(i2c: Result<I2cDriver<'a>, EspError>) -> Self {
        Self {
            i2c,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            backlight: LCD_NOBACKLIGHT,
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let display_function = LCD_4BITMODE | LCD_2LINE | LCD_5X8DOTS;
        FreeRtos::delay_ms(50);

        self.expander_write(self.backlight)?;

        FreeRtos::delay_ms(1000);

        for _ in 0..3 {
            self.write4bits((0x03 << 4) | self.backlight)?;
            FreeRtos::delay_ms(5);
        }

        self.write4bits((0x02 << 4) | self.backlight)?;

        self.send(LCD_FUNCTIONSET | display_function, 0x0)?;

        self.display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.display_on()?;
        self.clear()?;

        self.display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
        self.send(LCD_ENTRYMODESET | self.display_mode, 0x0)?;

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

    fn display_on(&mut self) -> anyhow::Result<()> {
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

    fn clear(&self) -> anyhow::Result<()> {
        // Implement the method to clear the display
        Ok(())
    }
}
