#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod consts;

use crate::consts::*;
use esp_idf_hal::delay::{Ets, BLOCK};
use esp_idf_hal::i2c::*;
use esp_idf_hal::sys::EspError;

/// Represents an LCD display connected via I2C.
///
/// The `Lcd` struct encapsulates the state and functionality for controlling an LCD display.
/// It provides methods for initializing the display, controlling the backlight, setting the cursor,
/// printing characters and strings, and more.
///
/// # Fields
///
/// * `i2c` - A result containing an `I2cDriver` or an `EspError`.
/// * `cols` - The number of columns in the LCD.
/// * `rows` - The number of rows in the LCD.
/// * `display_mode` - The display mode settings.
/// * `display_control` - The display control settings.
/// * `backlight` - The backlight state.
/// * `current_line` - The current line position of the cursor.
pub struct Lcd<'a> {
    i2c: Result<I2cDriver<'a>, EspError>,
    cols: u8,
    rows: u8,
    display_mode: u8,
    display_control: u8,
    backlight: u8,
    current_line: u8,
}

impl<'a> Lcd<'a> {
    /// Creates a new `Lcd` instance.
    ///
    /// # Arguments
    ///
    /// * `i2c` - A result containing an `I2cDriver` or an `EspError`.
    /// * `cols` - The number of columns in the LCD.
    /// * `rows` - The number of rows in the LCD.
    ///
    /// # Returns
    ///
    /// A new `Lcd` instance.
    pub fn new(i2c: Result<I2cDriver<'a>, EspError>, cols: u8, rows: u8) -> Self {
        Self {
            i2c,
            cols,
            rows,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            backlight: LCD_NOBACKLIGHT,
            current_line: 0,
        }
    }

    /// Initializes the LCD display.
    ///
    /// This function sets up the LCD display by configuring the display function,
    /// writing initial commands, and setting the display mode and control settings.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the initialization is successful.
    /// * `Err(anyhow::Error)` - If there is an error during initialization.
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

    /// Turns on the LCD display.
    ///
    /// This function sets the display control bit to turn on the display and sends the command to the LCD.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the display is successfully turned on.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn display_on(&mut self) -> anyhow::Result<()> {
        self.display_control |= LCD_DISPLAYON;
        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Turns off the LCD display.
    ///
    /// This function clears the display control bit to turn off the display and sends the command to the LCD.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the display is successfully turned off.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn display_off(&mut self) -> anyhow::Result<()> {
        self.display_control &= !LCD_DISPLAYON;
        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Turns on the LCD backlight.
    ///
    /// This function sets the backlight bit and writes the value to the expander.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to write to the expander.
    pub fn backlight_on(&mut self) -> anyhow::Result<()> {
        self.backlight = LCD_BACKLIGHT;
        self.expander_write(self.backlight)?;
        Ok(())
    }

    /// Turns off the LCD backlight.
    ///
    /// This function clears the backlight bit and writes the value to the expander.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to write to the expander.
    pub fn backlight_off(&mut self) -> anyhow::Result<()> {
        self.backlight = LCD_NOBACKLIGHT;
        self.expander_write(self.backlight)?;
        Ok(())
    }

    /// Clears the LCD display.
    ///
    /// This function sends the `LCD_CLEARDISPLAY` command to the LCD, waits for the command to complete,
    /// and resets the `current_line` to 0.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the display is successfully cleared.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.send(LCD_CLEARDISPLAY, 0x0)?;
        Ets::delay_us(2000);
        self.current_line = 0;
        Ok(())
    }

    /// Sets the cursor to a specific column and row on the LCD.
    ///
    /// # Arguments
    ///
    /// * `col` - The column position (0-indexed).
    /// * `row` - The row position (0-indexed).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor is successfully set.
    /// * `Err(anyhow::Error)` - If the row is out of bounds or the number of rows is invalid.
    pub fn set_cursor(&mut self, col: u8, row: u8) -> anyhow::Result<()> {
        if row >= self.rows {
            return Err(anyhow::anyhow!("Row out of bounds"));
        }

        let row_offsets: &[u8] = match self.rows {
            1 => &[0x00],
            2 => &[0x00, 0x40],
            4 => &[0x00, 0x40, 0x14, 0x54],
            _ => return Err(anyhow::anyhow!("Invalid number of rows")),
        };

        let cmd = LCD_SETDDRAMADDR | (col + row_offsets[row as usize]);
        self.send(cmd, 0x0)?;
        self.current_line = row;
        Ok(())
    }

    /// Controls the cursor visibility on the LCD.
    ///
    /// This function sets or clears the cursor control bit to turn the cursor on or off
    /// and sends the command to the LCD.
    ///
    /// # Arguments
    ///
    /// * `on` - A boolean indicating whether to turn the cursor on (`true`) or off (`false`).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor visibility is successfully changed.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn cursor(&mut self, on: bool) -> anyhow::Result<()> {
        if on {
            self.display_control |= LCD_CURSORON;
        } else {
            self.display_control &= !LCD_CURSORON;
        }

        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Controls the blinking of the cursor on the LCD.
    ///
    /// This function sets or clears the blink control bit to turn the cursor blinking on or off
    /// and sends the command to the LCD.
    ///
    /// # Arguments
    ///
    /// * `on` - A boolean indicating whether to turn the cursor blinking on (`true`) or off (`false`).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor blinking is successfully changed.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn blink(&mut self, on: bool) -> anyhow::Result<()> {
        if on {
            self.display_control |= LCD_BLINKON;
        } else {
            self.display_control &= !LCD_BLINKON;
        }

        let cmd = LCD_DISPLAYCONTROL | self.display_control;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Prints a single character to the LCD.
    ///
    /// # Arguments
    ///
    /// * `ch` - The character to print.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the character is successfully printed.
    /// * `Err(anyhow::Error)` - If there is an error while sending the character.
    pub fn print(&mut self, ch: char) -> anyhow::Result<()> {
        let data = ch as u8;
        self.send(data, RS)?;
        Ok(())
    }

    /// Prints a string to the LCD.
    ///
    /// # Arguments
    ///
    /// * `str` - The string to print.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the string is successfully printed.
    /// * `Err(anyhow::Error)` - If there is an error while printing any character.
    pub fn print_str(&mut self, str: &str) -> anyhow::Result<()> {
        for ch in str.chars() {
            self.print(ch)?
        }
        Ok(())
    }

    /// Prints a long string to the LCD, wrapping text to the next line if necessary.
    /// The text will be printed starting from the home position (0,0).
    ///
    /// # Arguments
    ///
    /// * `str` - The long string to print.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the string is successfully printed.
    /// * `Err(anyhow::Error)` - If there is an error while printing any character or setting the cursor.
    pub fn print_long_str(&mut self, str: &str) -> anyhow::Result<()> {
        let mut col = 0;
        let mut row = 0;
        self.set_cursor(col, row)?;

        for (i, ch) in str.chars().enumerate() {
            col = (i as u8) % self.cols;
            row = (i as u8) / self.cols;

            if row >= self.rows {
                row = 0;
            }

            self.set_cursor(col, row)?;
            self.print(ch)?;
        }

        Ok(())
    }

    /// Controls the autoscroll feature of the LCD.
    ///
    /// This function enables or disables the autoscroll feature, which causes the display to automatically
    /// scroll the text when new characters are printed.
    ///
    /// # Arguments
    ///
    /// * `on` - A boolean indicating whether to enable (`true`) or disable (`false`) autoscroll.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the autoscroll setting is successfully changed.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn autoscroll(&mut self, on: bool) -> anyhow::Result<()> {
        if on {
            self.display_mode |= LCD_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode &= !LCD_ENTRYSHIFTINCREMENT;
        }

        let cmd = LCD_ENTRYMODESET | self.display_mode;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Scrolls the display to the left.
    ///
    /// This function shifts the entire display to the left by one position.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the display is successfully scrolled.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn scroll_left(&mut self) -> anyhow::Result<()> {
        let cmd = LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Scrolls the display to the right.
    ///
    /// This function shifts the entire display to the right by one position.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the display is successfully scrolled.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn scroll_right(&mut self) -> anyhow::Result<()> {
        let cmd = LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Sets the text direction to left-to-right.
    ///
    /// This function sets the display mode to left-to-right text entry and sends the command to the LCD.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the text direction is successfully set.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn left_to_right(&mut self) -> anyhow::Result<()> {
        self.display_mode |= LCD_ENTRYLEFT;
        let cmd = LCD_ENTRYMODESET | self.display_mode;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Sets the text direction to right-to-left.
    ///
    /// This function sets the display mode to right-to-left text entry and sends the command to the LCD.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the text direction is successfully set.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn right_to_left(&mut self) -> anyhow::Result<()> {
        self.display_mode &= !LCD_ENTRYLEFT;
        let cmd = LCD_ENTRYMODESET | self.display_mode;
        self.send(cmd, 0x0)?;
        Ok(())
    }

    /// Returns the cursor to the home position (0,0).
    ///
    /// This function sends the `LCD_RETURNHOME` command to the LCD, which moves the cursor
    /// to the home position (0,0) and waits for the command to complete.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor is successfully moved to the home position.
    /// * `Err(anyhow::Error)` - If there is an error while sending the command.
    pub fn home(&mut self) -> anyhow::Result<()> {
        self.send(LCD_RETURNHOME, 0x0)?;
        Ets::delay_us(2000);
        Ok(())
    }

    /// Moves the cursor to the next line on the LCD.
    ///
    /// This function increments the `current_line` and sets the cursor to the beginning
    /// of the next line. If the display has only one row, it returns an error. If the
    /// `current_line` exceeds the number of rows, it wraps around to the first row.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor is successfully moved to the next line.
    /// * `Err(anyhow::Error)` - If the display has only one row.
    pub fn next_line(&mut self) -> anyhow::Result<()> {
        if self.rows == 1 {
            return Err(anyhow::anyhow!("Next line not supported on 1 row display"));
        }
        if self.current_line >= self.rows {
            self.current_line = 0;
        } else {
            self.current_line += 1;
        }
        self.set_cursor(0, self.current_line)?;
        Ok(())
    }

    /// Creates a custom character in the LCD's CGRAM (Character Generator RAM).
    ///
    /// # Arguments
    ///
    /// * `location` - The location in CGRAM to store the custom character (0-7).
    /// * `charmap` - A slice containing the character map (8 bytes).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the custom character is successfully created.
    /// * `Err(anyhow::Error)` - If the location is out of bounds or there is an error while sending the data.
    pub fn create_custom_chars(&mut self, location: u8, charmap: &[u8]) -> anyhow::Result<()> {
        if location > 7 {
            return Err(anyhow::anyhow!("Custom character location out of bounds"));
        }
        self.send(LCD_SETCGRAMADDR | (location << 3), 0x0)?;
        for i in 0..8 {
            self.send(charmap[i], RS)?;
        }
        Ok(())
    }

    fn expander_write(&mut self, data: u8) -> anyhow::Result<()> {
        let bytes = [0, data];
        self.i2c
            .as_mut()
            .unwrap()
            .write(LCD_ADDRESS, &bytes, BLOCK)
            .expect("Failed to write to expander");
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
