use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::*,
    peripheral::Peripheral,
    delay::Ets,
};
use esp_idf_svc::sys::EspError;

pub struct SevenSegment<'a> {
    clk: PinDriver<'a, AnyIOPin, InputOutput>,
    dio: PinDriver<'a, AnyIOPin, InputOutput>,
}

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

    pub fn start(&mut self) -> Result<()> {
        self.dio.set_low()?;
        self.bit_delay();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.dio.set_low()?;
        self.bit_delay();
        self.clk.set_high()?;
        self.bit_delay();
        self.dio.set_high()?;
        self.bit_delay();

        Ok(())
    }

    pub fn bit_delay(&self) {
        Ets::delay_us(100u32);
    }

    pub fn set_segments(&mut self, segments: [u8; 4]) -> Result<()> {
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

