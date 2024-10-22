# LCD I2C Driver for ESP32

This Rust crate provides a library for controlling various LCD displays (e.g., 16x2, 20x4) using the I2C protocol with the ESP32 microcontroller. It supports common functionalities such as cursor manipulation, display control, custom character creation, and more.

## Features

- Multi-size LCD display support (16x2, 20x4, etc.).
- I2C communication with ESP32.
- Basic display control functions: clear, home, turn on/off, backlight control.
- Custom character creation (e.g., emojis, graphics).
- Cursor management: move cursor, enable/disable cursor blink.
- Print text, including handling long strings.
- Line management for smooth text flow across rows.

## Requirements

- Rust toolchain
- ESP32 development environment
- `anyhow` for error handling
- `esp-idf-sys` for ESP32 system support

## Installation

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
lcd_i2c_rs = "1.0.0"
```
Ensure that you have setup the [esp-idf](https://github.com/esp-rs/esp-idf-template) toolchain for Rust Development on ESP32.

## Example

```rust
use lcd_i2c_rs::Lcd;
use esp_idf_hal::i2c::*;

let i2c = I2cDriver::new(/* params */).unwrap();
let mut lcd = Lcd::new(i2c, 16, 2); // Initialize for a 16x2 LCD display
lcd.init().unwrap();
lcd.print("Hello, World!").unwrap();
lcd.clear().unwrap();
lcd.set_cursor(0, 1).unwrap(); // Move to first column of the second row

let smiley = [
    0b00000,
    0b01010,
    0b00000,
    0b00000,
    0b10001,
    0b01110,
    0b00000,
];
lcd.create_custom_chars(0, &smiley).unwrap();
lcd.print("\0").unwrap(); // Print the custom character

lcd.cursor(true).unwrap(); // Enable the cursor
lcd.blink(true).unwrap();  // Enable blinking

lcd.print_long_str("This string is longer than one line and will wrap around.").unwrap();
```

## API Documentation

The API documentation can be found [here](https://docs.rs/lcd_i2c_rs/1.0.0/lcd_i2c_rs/).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you'd like to improve this library.

### Guidelines

- Follow the Rust API design guidelines.
- Ensure compatibility with common LCD displays.
- Document all public functions and structs.

[//]: # (- Write tests for new functionality.)

## License

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

