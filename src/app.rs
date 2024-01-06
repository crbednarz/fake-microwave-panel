use anyhow::Result;
use esp_idf_svc::hal::{
    peripherals::Peripherals,
    timer::*,
    gpio::{
        AnyInputPin,
        Input,
        PinDriver,
        Level, AnyIOPin,
    },
    timer::config::Config,
    delay::FreeRtos, peripheral::Peripheral,
};
use esp_idf_svc::sys::{
    esp_deep_sleep_start,
};
use crate::seven_segment::SevenSegment;
use crate::keypad::Keypad;
use crate::speaker::Speaker;
use awedio::{
    sounds::MemorySound,
    Sound,
};
use std::sync::Arc;

const MICROWAVE_BEEP_WAV: &[u8] = include_bytes!("./assets/beep.wav");
const MICROWAVE_START_WAV: &[u8] = include_bytes!("./assets/start.wav");
const MICROWAVE_RUNNING_WAV: &[u8] = include_bytes!("./assets/microwave.wav");

const DISPLAY_DIGITS: [u8;11] = [
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
    0b00000000,
];

#[derive(Clone, Copy)]
enum Mode {
    Idle,
    UserInput,
    Running{seconds: u8, minutes: u8},
    Done,
    Paused{seconds: u8, minutes: u8},
    Sleep,
}

struct SoundPack {
    beep: Arc<Vec<i16>>,
    start: Arc<Vec<i16>>,
    running: Arc<Vec<i16>>,
}

impl SoundPack {
    fn new() -> Self {
        Self {
            beep: Self::convert_wav_to_samples(&MICROWAVE_BEEP_WAV),
            start: Self::convert_wav_to_samples(&MICROWAVE_START_WAV),
            running: Self::convert_wav_to_samples(&MICROWAVE_RUNNING_WAV),
        }
    }

    fn convert_wav_to_samples(wav: &[u8]) -> Arc<Vec<i16>> {
        let wav = &wav[44..];
        let mut samples = vec![0i16; wav.len()/2];
        for i in 0..samples.len() {
            samples[i] = (wav[i * 2] as i16) | ((wav[i * 2 + 1] as i16) << 8);
        }
        Arc::new(samples)
    }

    fn beep_sound(&self) -> Box<MemorySound> {
        Box::new(MemorySound::from_samples(self.beep.clone(), 1, 16000))
    }

    fn start_sound(&self) -> Box<MemorySound> {
        Box::new(MemorySound::from_samples(self.start.clone(), 1, 16000))
    }

    fn running_sound(&self) -> Box<dyn Sound> {
        let mut sound = MemorySound::from_samples(self.running.clone(), 1, 16000);
        sound.set_looping(true);
        Box::new(sound)
    }
}

struct App<'a> {
    display: SevenSegment<'a>,
    keypad: Keypad<'a>,
    speaker: Speaker,
    timer: TimerDriver<'a>,
    start_button: PinDriver<'a, AnyInputPin, Input>,
    stop_button: PinDriver<'a, AnyInputPin, Input>,
    door_switch: PinDriver<'a, AnyInputPin, Input>,
    sounds: SoundPack,
}

impl<'a> App<'a> {
    pub fn new(
        display: SevenSegment<'a>,
        keypad: Keypad<'a>,
        speaker: Speaker,
        timer: TimerDriver<'a>,
        start_button: PinDriver<'a, AnyInputPin, Input>,
        stop_button: PinDriver<'a, AnyInputPin, Input>,
        door_switch: PinDriver<'a, AnyInputPin, Input>,
    ) -> Self {
        Self {
            display,
            keypad,
            speaker,
            timer,
            start_button,
            stop_button,
            door_switch,
            sounds: SoundPack::new(),
        }
    }

    fn run(&mut self) -> Result<()> {
        self.timer.enable(true)?;
        let mut mode = Mode::Idle;
        loop {
            let next_mode = match mode {
                Mode::Idle => self.run_idle()?,
                Mode::UserInput => self.run_user_input()?,
                Mode::Running{seconds, minutes} => self.run_running(seconds, minutes)?,
                Mode::Done => self.run_done()?,
                Mode::Paused{seconds, minutes} => self.run_paused(seconds, minutes)?,
                Mode::Sleep => self.run_sleep()?,
            };
            mode = next_mode;
        }
    }

    fn run_idle(&mut self) -> Result<Mode> {
        self.display.set_segments([0b00000000; 4])?;
        let start_time = self.timer.counter()?;
        const TIMEOUT: u64 = 60;
        loop {
            let key = self.keypad.get_key()?;
            if key.is_some() {
                return Ok(Mode::UserInput);
            }
            let elapsed = self.timer.counter()? - start_time;
            let seconds_elapsed = elapsed / self.timer.tick_hz();
            if seconds_elapsed > TIMEOUT {
                return Ok(Mode::Sleep);
            }
            FreeRtos::delay_ms(50u32);
        }
    }


    fn run_user_input(&mut self) -> Result<Mode> {
        self.display.set_segments([0b00000000; 4])?;
        let start_time = self.timer.counter()?;
        const TIMEOUT: u64 = 60 * 5;

        let mut last_key = None;
        let mut digits = [10u8; 4];
        loop {
            let key = self.keypad.get_key()?;
            if key != last_key && key.is_some() {
                let mut digit = key.unwrap() + 1;
                if digit > 9 {
                    digit = 0;
                }
                if digits[1] == 10 {
                    digits = [digits[1], digits[2], digits[3], digit];
                }

                self.display.set_segments([
                    DISPLAY_DIGITS[digits[0] as usize],
                    DISPLAY_DIGITS[digits[1] as usize] | 0x80,
                    DISPLAY_DIGITS[digits[2] as usize],
                    DISPLAY_DIGITS[digits[3] as usize],
                ])?;
                self.speaker.play(self.sounds.beep_sound())?;
            }
            if self.start_button.get_level() == Level::Low && self.door_switch.get_level() == Level::Low {
                self.speaker.play(self.sounds.start_sound())?;
                let mut seconds = if digits[2] == 10 { 0 } else { digits[2] * 10 };
                seconds += if digits[3] == 10 { 0 } else { digits[3] };
                let mut minutes = if digits[0] == 10 { 0 } else { digits[0] * 10 };
                minutes += if digits[1] == 10 { 0 } else { digits[1] };
                return Ok(Mode::Running{seconds, minutes});
            }
            if self.stop_button.get_level() == Level::Low {
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Idle);
            }
            let elapsed = self.timer.counter()? - start_time;
            let seconds_elapsed = elapsed / self.timer.tick_hz();
            if seconds_elapsed > TIMEOUT {
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Idle);
            }
            FreeRtos::delay_ms(50u32);
            last_key = key;
        }
    }

    fn run_running(&mut self, seconds: u8, minutes: u8) -> Result<Mode> {
        let running_sound = self.sounds.running_sound();
        self.speaker.play(running_sound)?;
        let start_time = self.timer.counter()?;
        let mut last_seconds_elapsed = 0;
        let mut seconds = seconds;
        let mut minutes = minutes;
        loop {
            let elapsed = self.timer.counter()? - start_time;
            let seconds_elapsed = elapsed / self.timer.tick_hz();
            if seconds_elapsed != last_seconds_elapsed {
                if seconds == 0 {
                    minutes -= 1;
                    seconds = 59;
                } else {
                    seconds -= 1;
                }
                if minutes == 0 && seconds == 0 {
                    self.speaker.clear();
                    return Ok(Mode::Done);
                }
                let mut digits = [
                    minutes as usize / 10,
                    minutes as usize % 10,
                    seconds as usize / 10,
                    seconds as usize % 10,
                ];
                for i in 0..4 {
                    if digits[i] == 0 {
                        digits[i] = 10;
                    } else {
                        break;
                    }
                }
                
                self.display.set_segments([
                    DISPLAY_DIGITS[digits[0]],
                    DISPLAY_DIGITS[digits[1]] | 0x80,
                    DISPLAY_DIGITS[digits[2]],
                    DISPLAY_DIGITS[digits[3]],
                ])?;
                last_seconds_elapsed = seconds_elapsed;
            }
            if self.door_switch.get_level() == Level::High {
                self.speaker.clear();
                return Ok(Mode::Paused{seconds, minutes});
            }
            if self.stop_button.get_level() == Level::Low {
                self.speaker.clear();
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Idle);
            }

            FreeRtos::delay_ms(50u32);
        }
    }

    fn run_done(&mut self) -> Result<Mode> {
        for _ in 0..5 {
            self.speaker.play(self.sounds.beep_sound())?;
            self.display.set_segments([0b01111111, 0b01111001, 0b01111001, 0b01110011])?;
            FreeRtos::delay_ms(500u32);
            self.display.set_segments([0b00000000, 0b00000000, 0b00000000, 0b00000000])?;
            FreeRtos::delay_ms(500u32);
        }
        Ok(Mode::Idle)
    }

    fn run_paused(&mut self, seconds: u8, minutes: u8) -> Result<Mode> {
        let start_time = self.timer.counter()?;
        const TIMEOUT: u64 = 60 * 5;
        loop {
            if self.start_button.get_level() == Level::Low && self.door_switch.get_level() == Level::Low {
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Running{seconds, minutes});
            }
            if self.stop_button.get_level() == Level::Low {
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Idle);
            }
            let elapsed = self.timer.counter()? - start_time;
            let seconds_elapsed = elapsed / self.timer.tick_hz();
            if seconds_elapsed > TIMEOUT {
                self.speaker.play(self.sounds.beep_sound())?;
                return Ok(Mode::Idle);
            }
            FreeRtos::delay_ms(50u32);
        }
    }

    fn run_sleep(&mut self) -> Result<Mode> {
        Ok(Mode::Idle)
    }
}

pub fn run_app() -> Result<()> {
    let peripherals = Peripherals::take()?;
    let display = SevenSegment::new(peripherals.pins.gpio16, peripherals.pins.gpio17)?;
    let keypad = Keypad::new(
        peripherals.pins.gpio14, peripherals.pins.gpio25, peripherals.pins.gpio21,
        peripherals.pins.gpio26, peripherals.pins.gpio19, peripherals.pins.gpio22, peripherals.pins.gpio13,
    )?;
    let speaker = Speaker::new(
        peripherals.i2s0,
        peripherals.pins.gpio15, peripherals.pins.gpio23, peripherals.pins.gpio4,
    )?;
    let timer = TimerDriver::new(peripherals.timer00, &Config::default())?;
    let start_button = PinDriver::input(peripherals.pins.gpio34.into_ref().map_into::<AnyInputPin>())?;
    let stop_button = PinDriver::input(peripherals.pins.gpio35.into_ref().map_into::<AnyInputPin>())?;
    let door_switch = PinDriver::input(peripherals.pins.gpio39.into_ref().map_into::<AnyInputPin>())?;
    let mut light = PinDriver::output(peripherals.pins.gpio12.into_ref().map_into::<AnyIOPin>())?;
    light.set_level(Level::High)?;

    let mut app = App::new(
        display,
        keypad,
        speaker,
        timer,
        start_button,
        stop_button,
        door_switch,
    );

    app.run()
}
