use esp_idf_hal::delay::{FreeRtos, BLOCK};
use esp_idf_hal::i2c::*;

pub struct I2CLcd<I2C> {
    i2c: I2C,
    address: u8,
}

