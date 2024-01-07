use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::OutputPin,
    rmt::{
        config::{
            CarrierConfig,
            DutyPercent,
            Loop,
            TransmitConfig,
        },
        PinState,
        PulseTicks,
        Pulse,
        RmtChannel,
        TxRmtDriver,
        VariableLengthSignal, RmtTransmitConfig,
    },
    peripheral::Peripheral,
    units::FromValueType,
};
use esp_idf_svc::sys::EspError;

pub struct Remote<'d> {
    tx: TxRmtDriver<'d>,
}

impl<'d> Remote<'d> {
    pub fn new(
        channel: impl Peripheral<P = impl RmtChannel> + 'd,
        led: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self> {
        let carrier = CarrierConfig::new()
            .duty_percent(DutyPercent::new(50)?)
            .frequency(37000.Hz());
        let mut config = TransmitConfig::new()
            .carrier(Some(carrier));
        Ok(Self {
            tx: TxRmtDriver::new(
                channel,
                led,
                &mut config,
            )?,
        })
    }

    pub fn send_on(&mut self) -> Result<(), EspError> {
        let mut signal = VariableLengthSignal::new();
        signal.push([
            &high_pulse(9201)?,
            &low_pulse(4497)?,
            &high_pulse(587)?,
            &low_pulse(580)?,
            &high_pulse(586)?,
            &low_pulse(582)?,
            &high_pulse(613)?,
            &low_pulse(555)?,
            &high_pulse(587)?,
            &low_pulse(581)?,
            &high_pulse(588)?,
            &low_pulse(579)?,
            &high_pulse(587)?,
            &low_pulse(583)?,
            &high_pulse(614)?,
            &low_pulse(555)?,
            &high_pulse(586)?,
            &low_pulse(582)?,
            &high_pulse(614)?,
            &low_pulse(1634)?,
            &high_pulse(588)?,
            &low_pulse(1659)?,
            &high_pulse(615)?,
            &low_pulse(1632)?,
            &high_pulse(589)?,
            &low_pulse(1657)?,
            &high_pulse(615)?,
            &low_pulse(1634)?,
            &high_pulse(616)?,
            &low_pulse(1630)?,
            &high_pulse(617)?,
            &low_pulse(1631)?,
            &high_pulse(616)?,
            &low_pulse(1631)?,
            &high_pulse(614)?,
            &low_pulse(1634)?,
            &high_pulse(613)?,
            &low_pulse(556)?,
            &high_pulse(611)?,
            &low_pulse(1636)?,
            &high_pulse(614)?,
            &low_pulse(553)?,
            &high_pulse(616)?,
            &low_pulse(553)?,
            &high_pulse(615)?,
            &low_pulse(554)?,
            &high_pulse(615)?,
            &low_pulse(1633)?,
            &high_pulse(614)?,
            &low_pulse(554)?,
            &high_pulse(614)?,
            &low_pulse(555)?,
            &high_pulse(615)?,
            &low_pulse(1632)?,
            &high_pulse(612)?,
            &low_pulse(556)?,
            &high_pulse(615)?,
            &low_pulse(1632)?,
            &high_pulse(612)?,
            &low_pulse(1635)?,
            &high_pulse(616)?,
            &low_pulse(1631)?,
            &high_pulse(611)?,
            &low_pulse(557)?,
            &high_pulse(614)?,
            &low_pulse(1633)?,
            &high_pulse(614)?,
        ])?;
        self.tx.start(signal)
    }

    pub fn send_off(&mut self) -> Result<(), EspError> {
        let mut signal = VariableLengthSignal::new();
        signal.push([
            &high_pulse(9204)?,
            &low_pulse(4512)?,
            &high_pulse(578)?,
            &low_pulse(591)?,
            &high_pulse(579)?,
            &low_pulse(592)?,
            &high_pulse(579)?,
            &low_pulse(591)?,
            &high_pulse(580)?,
            &low_pulse(589)?,
            &high_pulse(582)?,
            &low_pulse(590)?,
            &high_pulse(580)?,
            &low_pulse(591)?,
            &high_pulse(580)?,
            &low_pulse(592)?,
            &high_pulse(579)?,
            &low_pulse(591)?,
            &high_pulse(582)?,
            &low_pulse(1669)?,
            &high_pulse(579)?,
            &low_pulse(1669)?,
            &high_pulse(581)?,
            &low_pulse(1669)?,
            &high_pulse(581)?,
            &low_pulse(1669)?,
            &high_pulse(580)?,
            &low_pulse(1669)?,
            &high_pulse(580)?,
            &low_pulse(1670)?,
            &high_pulse(580)?,
            &low_pulse(1669)?,
            &high_pulse(579)?,
            &low_pulse(1671)?,
            &high_pulse(579)?,
            &low_pulse(1669)?,
            &high_pulse(580)?,
            &low_pulse(1669)?,
            &high_pulse(580)?,
            &low_pulse(1669)?,
            &high_pulse(581)?,
            &low_pulse(591)?,
            &high_pulse(580)?,
            &low_pulse(592)?,
            &high_pulse(578)?,
            &low_pulse(592)?,
            &high_pulse(579)?,
            &low_pulse(1670)?,
            &high_pulse(580)?,
            &low_pulse(592)?,
            &high_pulse(579)?,
            &low_pulse(590)?,
            &high_pulse(580)?,
            &low_pulse(592)?,
            &high_pulse(578)?,
            &low_pulse(593)?,
            &high_pulse(577)?,
            &low_pulse(1672)?,
            &high_pulse(577)?,
            &low_pulse(1672)?,
            &high_pulse(578)?,
            &low_pulse(1673)?,
            &high_pulse(577)?,
            &low_pulse(592)?,
            &high_pulse(578)?,
            &low_pulse(1673)?,
            &high_pulse(577)?,
        ])?;
        self.tx.start(signal)
    }
}

fn high_pulse(ticks: u16) -> Result<Pulse, EspError> {
    Ok(Pulse::new(PinState::High, PulseTicks::new(ticks)?))
}

fn low_pulse(ticks: u16) -> Result<Pulse, EspError> {
    Ok(Pulse::new(PinState::Low, PulseTicks::new(ticks)?))
}
