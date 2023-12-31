use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::*,
    peripheral::Peripheral,
};
use esp_idf_svc::sys::EspError;

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

    pub fn get_key(&mut self) -> Result<Option<u8>> {
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
