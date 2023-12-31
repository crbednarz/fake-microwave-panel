use anyhow::Result;
use esp_idf_svc::hal::{
    i2s::{
        I2s,
        I2sDriver,
        I2sTx,
        config::{
            Config,
            SlotMode,
            StdClkConfig,
            StdSlotConfig,
            StdGpioConfig,
            StdConfig,
            DataBitWidth
        },
    },
    gpio::*,
    peripheral::Peripheral,
};
use awedio::{
    manager::Manager,
    Sound,
};
use awedio_esp32::Esp32Backend;
use esp_idf_svc::sys::EspError;

pub struct Speaker {
    manager: Manager,
}

impl Speaker {
    pub fn new(
        i2s_peripheral: impl Peripheral<P = impl I2s> + 'static,
        bclk: impl Peripheral<P = impl InputPin + OutputPin> + 'static,
        dout: impl Peripheral<P = impl OutputPin> + 'static,
        ws: impl Peripheral<P = impl InputPin + OutputPin> + 'static,
    ) -> Result<Self, EspError> {
        let mclk = AnyIOPin::none();

        let std_config = StdConfig::new(
            Config::default(),
            StdClkConfig::from_sample_rate_hz(16000),
            StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Stereo),
            StdGpioConfig::default(),
        );
        let i2s = I2sDriver::<I2sTx>::new_std_tx(
            i2s_peripheral,
            &std_config,
            bclk,
            dout,
            mclk,
            ws,
        )?;
        let backend = Esp32Backend::with_defaults(
            i2s,
            2,
            16000,
            128,
        );
        let manager = backend.start();
        Ok(Self {manager})
    }

    pub fn play(&mut self, sound: Box<dyn Sound>) -> Result<()> {
        self.manager.play(sound);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.manager.clear();
    }
}
