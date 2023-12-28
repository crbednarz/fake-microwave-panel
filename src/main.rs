pub mod seven_segment;
pub mod keypad;

use anyhow::Result;
use esp_idf_svc::hal::{
    peripherals::Peripherals,
    delay::FreeRtos,
};
use crate::seven_segment::SevenSegment;
use crate::keypad::Keypad;

fn test() -> Result<()> {
    let peripherals = Peripherals::take()?;
    let mut ss = SevenSegment::new(peripherals.pins.gpio22, peripherals.pins.gpio21)?;
    let mut keypad = Keypad::new(
        peripherals.pins.gpio32, peripherals.pins.gpio19, peripherals.pins.gpio25,
        peripherals.pins.gpio18, peripherals.pins.gpio27, peripherals.pins.gpio26, peripherals.pins.gpio33,
    )?;
    FreeRtos::delay_ms(1000u32);
    let digits = [
        0b00111111,
        0b00000110,
        0b01011011,
        0b01001111,
        0b01100110,
        0b01101101,
        0b01111101,
        0b00000111,
        0b01111111,
        0b01101111,
    ];
    /*
    for i in 0..255 {
        let minutes = i / 60;
        let seconds = i % 60;
        let mut minutes_tens = minutes / 10;
        let minutes_ones = minutes % 10;
        let seconds_tens = seconds / 10;
        let seconds_ones = seconds % 10;
        let key = keypad.get_key();
        if let Ok(Some(key)) = key {
            minutes_tens = key % 10;
        }

        ss.set_segments([
            digits[minutes_tens as usize],
            digits[minutes_ones as usize] | 0x80,
            digits[seconds_tens as usize],
            digits[seconds_ones as usize],
        ])?;
        FreeRtos::delay_ms(1000u32);
    }*/
    
    loop {
        let key = keypad.get_key();
        if let Ok(Some(key)) = key {
            let mut key = key + 1;
            if key > 9 {
                key = 0;
            }
            ss.set_segments([
                digits[0],
                digits[0],
                digits[0],
                digits[key as usize],
            ])?;
        }
        FreeRtos::delay_ms(100u32);
    }

}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    test().unwrap();
    log::info!("Hello, world! 2");
}
