use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::*,
    peripherals::Peripherals,
    peripheral::Peripheral,
    delay::{FreeRtos, Ets},
};
use esp_idf_svc::sys::EspError;

pub struct SevenSegment<'a> {
    clk: PinDriver<'a, AnyIOPin, InputOutput>,
    dio: PinDriver<'a, AnyIOPin, InputOutput>,
}

// Add new method to SevenSegment
impl<'d> SevenSegment<'d> {
    pub fn new(
        clk: impl Peripheral<P = impl IOPin> + 'd,
        dio: impl Peripheral<P = impl IOPin> + 'd,
    ) -> Result<Self, EspError> {
        let clk = clk.into_ref();
        let dio = dio.into_ref();
        let mut clk_driver = PinDriver::input_output_od(clk.map_into::<AnyIOPin>())?;
        let mut dio_driver = PinDriver::input_output_od(dio.map_into::<AnyIOPin>())?;

        clk_driver.set_high()?;
        dio_driver.set_high()?;
        clk_driver.set_pull(Pull::Up)?;
        dio_driver.set_pull(Pull::Up)?;
        Ok(Self {
            clk: clk_driver,
            dio: dio_driver,
        })
    }

    fn start(&mut self) -> Result<()> {
        self.dio.set_low()?;
        self.bit_delay();
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.dio.set_low()?;
        self.bit_delay();
        self.clk.set_high()?;
        self.bit_delay();
        self.dio.set_high()?;
        self.bit_delay();

        Ok(())
    }

    fn bit_delay(&self) {
        Ets::delay_us(100u32);
    }

    fn set_segments(&mut self, segments: [u8; 4]) -> Result<()> {
        self.start()?;
        self.write_byte(0x40)?;
        self.stop()?;

        self.start()?;
        self.write_byte(0xC0)?;
        for segment in segments.iter() {
            self.write_byte(*segment)?;
        }
        self.stop()?;

        self.start()?;
        self.write_byte(0x8F)?;
        self.stop()?;

        Ok(())
    }

    fn write_byte(&mut self, data: u8) -> Result<()> {
        let mut data = data;
        for _ in 0..8 {
            self.clk.set_low()?;
            self.bit_delay();
            
            if data & 1 != 0 {
                self.dio.set_high()?;
            } else {
                self.dio.set_low()?;
            }
            self.bit_delay();
            self.clk.set_high()?;
            self.bit_delay();
            data >>= 1;
        }

        self.clk.set_low()?;
        self.dio.set_high()?;
        self.bit_delay();

        self.clk.set_high()?;
        self.bit_delay();

        if self.dio.get_level() == Level::Low {
            self.dio.set_low()?;
        }

        self.bit_delay();
        self.clk.set_low()?;
        self.bit_delay();
        Ok(())
    }
}

pub struct Keypad<'a> {
    cols: [PinDriver<'a, AnyIOPin, InputOutput>; 3],
    rows: [PinDriver<'a, AnyIOPin, InputOutput>; 4],
}

impl<'d> Keypad<'d> {
    pub fn new(
        c1: impl Peripheral<P = impl IOPin> + 'd,
        c2: impl Peripheral<P = impl IOPin> + 'd,
        c3: impl Peripheral<P = impl IOPin> + 'd,
        r1: impl Peripheral<P = impl IOPin> + 'd,
        r2: impl Peripheral<P = impl IOPin> + 'd,
        r3: impl Peripheral<P = impl IOPin> + 'd,
        r4: impl Peripheral<P = impl IOPin> + 'd,
    ) -> Result<Self, EspError> {
        let mut col_drivers = [
            PinDriver::input_output_od(c1.into_ref().map_into::<AnyIOPin>())?,
            PinDriver::input_output_od(c2.into_ref().map_into::<AnyIOPin>())?,
            PinDriver::input_output_od(c3.into_ref().map_into::<AnyIOPin>())?,
        ];
        let mut row_drivers = [
            PinDriver::input_output_od(r1.into_ref().map_into::<AnyIOPin>())?,
            PinDriver::input_output_od(r2.into_ref().map_into::<AnyIOPin>())?,
            PinDriver::input_output_od(r3.into_ref().map_into::<AnyIOPin>())?,
            PinDriver::input_output_od(r4.into_ref().map_into::<AnyIOPin>())?,
        ];

        for col in col_drivers.iter_mut() {
            col.set_pull(Pull::Up)?;
            col.set_high()?;
        }

        for row in row_drivers.iter_mut() {
            row.set_pull(Pull::Up)?;
            row.set_high()?;
        }


        Ok(Self{ cols: col_drivers, rows: row_drivers })
    }

    fn get_key(&mut self) -> Result<Option<u8>> {
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_low()?;
            for (j, col) in self.cols.iter_mut().enumerate() {
                if col.get_level() == Level::Low {
                    row.set_high()?;
                    return Ok(Some((i * 3 + j) as u8));
                }
            }
            row.set_high()?;
        }
        Ok(None)
    }
}

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
