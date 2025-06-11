#![doc = include_str!("../README.md")]
#![no_std]
#![allow(static_mut_refs)]

use embassy_stm32::{
    Peripherals,
    adc::{Adc, AdcChannel},
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{
        ADC1, EXTI1, EXTI2, EXTI4, EXTI5, EXTI8, EXTI10, EXTI12, EXTI13, EXTI14, EXTI15, PA0, PA4,
        PA5, PA8, PA15, PB0, PB1, PB4, PC0, PD0, PD1, PD2, PD3, PD4, PD5, PD8, PD9, PD10, PD12,
        PD13, PD14, PD15, PE1, PE2, PE5, PE9, PE10, PE11, PE12, PE13, PE14, PE15, TIM1, TIM2, TIM3,
        USART2,
    },
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm},
    usart::{BufferedInterruptHandler, BufferedUart, Config, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, once_lock::OnceLock};
use embassy_tmc::tmc2209::TMC2209AsyncUart;

use crate::components::{
    adc::BUDDY_ADC1,
    buzzer::{BuddyBuzzer, Buzzer},
    fan::{BuddyFan, Fan},
    filament_sensor::{BuddyFilamentSensor, FilamentSensor},
    heater::{BuddyHeater, Heater},
    pinda::{BuddyPinda, Pinda},
    rotary_button::{BuddyRotaryButton, RotaryButton},
    rotary_encoder::{BuddyRotaryEncoder, RotaryEncoder},
    thermistor::{BuddyThermistor, Thermistor},
};

pub mod components;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

static mut STEPPER_TX: [u8; 16] = [0u8; 16];
static mut STEPPER_RX: [u8; 16] = [0u8; 16];
static STEPPER_USART: OnceLock<Mutex<ThreadModeRawMutex, BufferedUart<'static>>> = OnceLock::new();

pub struct BuddyFans<'a> {
    pub fan_0: BuddyFan<'a>,
    pub fan_1: BuddyFan<'a>,
}

pub struct BuddyThermistors<'a> {
    pub bed: BuddyThermistor<'a>,
    pub board: BuddyThermistor<'a>,
    pub hotend: BuddyThermistor<'a>,
}

pub struct BuddyHeaters<'a> {
    pub bed: BuddyHeater<'a>,
    pub hotend: BuddyHeater<'a>,
}

pub type BuddyStepperInterruptDia<'a> =
    TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'static>>;
pub type BuddyStepperInputDia<'a> =
    TMC2209AsyncUart<'a, Output<'a>, Input<'a>, ThreadModeRawMutex, BufferedUart<'static>>;

pub struct BuddySteppers<'a> {
    pub x: BuddyStepperInterruptDia<'a>,
    pub y: BuddyStepperInterruptDia<'a>,
    pub z: BuddyStepperInterruptDia<'a>,
    pub e: BuddyStepperInputDia<'a>,
}

pub struct Board<'a> {
    pub buzzer: BuddyBuzzer<'a>,
    pub pinda_sensor: BuddyPinda<'a>,
    pub filament_sensor: BuddyFilamentSensor<'a>,
    pub rotary_button: BuddyRotaryButton<'a>,
    pub rotary_encoder: BuddyRotaryEncoder<'a>,
    pub heaters: BuddyHeaters<'a>,
    pub thermistors: BuddyThermistors<'a>,
    pub fans: BuddyFans<'a>,
    pub steppers: BuddySteppers<'a>,
}

impl<'a> Board<'a> {
    pub async fn new(p: Peripherals) -> Self {
        // init statics
        Self::init_adc1(p.ADC1);

        // init components
        let buzzer = Self::init_buzzer(p.PA0, p.TIM2);
        let pinda_sensor = Self::init_pinda(p.PA8, p.EXTI8);
        let hotend = Self::init_hotend_thermistor(p.PC0);
        let bed = Self::init_bed_thermistor(p.PA4);
        let board = Self::init_board_thermistor(p.PA5);
        let filament_sensor = Self::init_filament_sensor(p.PB4, p.EXTI4);
        let rotary_button = Self::init_rotary_button(p.PE12, p.EXTI12);
        let rotary_encoder = Self::init_rotary_encoder(p.PE13, p.EXTI13, p.PE15, p.EXTI15);
        let heaters = Self::init_heaters(p.PB0, p.PB1, p.TIM3);
        let thermistors = BuddyThermistors { hotend, bed, board };
        let fans = Self::init_fans(p.PE11, p.PE10, p.EXTI10, p.PE9, p.PE14, p.EXTI14, p.TIM1);

        // Stepper motors
        Self::init_stepper_usart(p.USART2, p.PD5);
        let x_stepper = Self::init_x_stepper(p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2).await;
        let y_stepper = Self::init_y_stepper(p.PD14, p.PD13, p.PD12, p.PE1, p.EXTI1).await;
        let z_stepper = Self::init_z_stepper(p.PD2, p.PD4, p.PD15, p.PE5, p.EXTI5).await;
        let e_stepper = Self::init_e_stepper(p.PD10, p.PD9, p.PD8, p.PA15).await;

        let steppers = BuddySteppers {
            x: x_stepper,
            y: y_stepper,
            z: z_stepper,
            e: e_stepper,
        };

        Self {
            buzzer,
            pinda_sensor,
            filament_sensor,
            rotary_button,
            rotary_encoder,
            heaters,
            thermistors,
            fans,
            steppers,
        }
    }

    pub fn init_adc1(peripheral: ADC1) {
        let mut adc = BUDDY_ADC1.try_lock().unwrap();
        let p = Some(Adc::new(peripheral));
        *adc = p;
    }

    pub fn init_stepper_usart(
        usart: USART2,
        tx: PD5,
    ) {
        let config = Config::default();
        let usart = BufferedUart::new_half_duplex(
            usart,
            tx,
            Irqs,
            unsafe { &mut STEPPER_TX },
            unsafe { &mut STEPPER_RX },
            config,
            HalfDuplexReadback::Readback,
            HalfDuplexConfig::PushPull,
        )
        .unwrap();
        usart.set_baudrate(9_600).unwrap();
        if STEPPER_USART.init(Mutex::new(usart)).is_err() {
            panic!();
        };
    }

    pub fn init_pinda(
        pin: PA8,
        ch: EXTI8,
    ) -> BuddyPinda<'a> {
        let exti = ExtiInput::new(pin, ch, Pull::None);
        Pinda::new(exti)
    }

    pub fn init_buzzer(
        ch: PA0,
        tim: TIM2,
    ) -> BuddyBuzzer<'a> {
        let buzzer = PwmPin::new_ch1(ch, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            Some(buzzer),
            None,
            None,
            None,
            khz(21),
            Default::default(),
        );
        let mut channels = pwm.split();
        channels.ch1.enable();
        Buzzer::new(channels.ch1)
    }

    pub fn init_bed_thermistor(bed: PA4) -> BuddyThermistor<'a> {
        const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
        Thermistor::new(
            &BUDDY_ADC1,
            bed.degrade_adc(),
            4_700.0,
            MAX_ADC_SAMPLE,
            4_092.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_board_thermistor(bed: PA5) -> BuddyThermistor<'a> {
        const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
        Thermistor::new(
            &BUDDY_ADC1,
            bed.degrade_adc(),
            4_700.0,
            MAX_ADC_SAMPLE,
            4_550.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_hotend_thermistor(bed: PC0) -> BuddyThermistor<'a> {
        const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
        Thermistor::new(
            &BUDDY_ADC1,
            bed.degrade_adc(),
            4_700.0,
            MAX_ADC_SAMPLE,
            4_267.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_fans(
        fan_0_pwm: PE11,
        fan_0_inp: PE10,
        fan_0_exti: EXTI10,
        fan_1_pwm: PE9,
        fan_1_inp: PE14,
        fan_1_exti: EXTI14,
        tim: TIM1,
    ) -> BuddyFans<'a> {
        let fan_0_pwm_pin = PwmPin::new_ch2(fan_0_pwm, OutputType::PushPull);
        let fan_1_pwm_pin = PwmPin::new_ch1(fan_1_pwm, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            Some(fan_1_pwm_pin),
            Some(fan_0_pwm_pin),
            None,
            None,
            khz(21),
            Default::default(),
        );
        let fan_0_exti = ExtiInput::new(fan_0_inp, fan_0_exti, Pull::Down);
        let fan_1_exti = ExtiInput::new(fan_1_inp, fan_1_exti, Pull::Down);
        let mut channels = pwm.split();
        channels.ch1.enable();
        channels.ch2.enable();
        let fan_0 = Fan::new(channels.ch2, fan_0_exti);
        let fan_1 = Fan::new(channels.ch1, fan_1_exti);
        BuddyFans { fan_0, fan_1 }
    }

    pub fn init_filament_sensor(
        pin: PB4,
        ch: EXTI4,
    ) -> BuddyFilamentSensor<'a> {
        let exti = ExtiInput::new(pin, ch, Pull::None);
        FilamentSensor::new(exti)
    }

    pub fn init_rotary_button(
        pin: PE12,
        ch: EXTI12,
    ) -> BuddyRotaryButton<'a> {
        let exti = ExtiInput::new(pin, ch, Pull::Up);
        RotaryButton::new(exti)
    }

    pub fn init_rotary_encoder(
        pin_a: PE13,
        ch_a: EXTI13,
        pin_b: PE15,
        ch_b: EXTI15,
    ) -> BuddyRotaryEncoder<'a> {
        let extia = ExtiInput::new(pin_a, ch_a, Pull::None);
        let extib = ExtiInput::new(pin_b, ch_b, Pull::None);
        RotaryEncoder::new(extia, extib)
    }

    pub fn init_heaters(
        bed: PB0,
        hotend: PB1,
        tim: TIM3,
    ) -> BuddyHeaters<'a> {
        let bed_pin = PwmPin::new_ch3(bed, OutputType::PushPull);
        let hotend_pin = PwmPin::new_ch4(hotend, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            None,
            None,
            Some(bed_pin),
            Some(hotend_pin),
            khz(21),
            Default::default(),
        );
        let channels = pwm.split();
        let bed = Heater::new(channels.ch3);
        let hotend = Heater::new(channels.ch4);
        BuddyHeaters { bed, hotend }
    }

    pub async fn init_x_stepper(
        en: PD3,
        step: PD1,
        dir: PD0,
        dia_pin: PE2,
        dia_ch: EXTI2,
    ) -> BuddyStepperInterruptDia<'a> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new(en, step, dir, dia, 1, STEPPER_USART.get().await).unwrap()
    }

    pub async fn init_y_stepper(
        en: PD14,
        step: PD13,
        dir: PD12,
        dia_pin: PE1,
        dia_ch: EXTI1,
    ) -> BuddyStepperInterruptDia<'a> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new(en, step, dir, dia, 3, STEPPER_USART.get().await).unwrap()
    }

    pub async fn init_z_stepper(
        en: PD2,
        step: PD4,
        dir: PD15,
        dia_pin: PE5,
        dia_ch: EXTI5,
    ) -> BuddyStepperInterruptDia<'a> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new(en, step, dir, dia, 0, STEPPER_USART.get().await).unwrap()
    }

    pub async fn init_e_stepper(
        en: PD10,
        step: PD9,
        dir: PD8,
        dia_pin: PA15,
    ) -> BuddyStepperInputDia<'a> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = Input::new(dia_pin, Pull::None);
        TMC2209AsyncUart::new(en, step, dir, dia, 2, STEPPER_USART.get().await).unwrap()
    }
}
