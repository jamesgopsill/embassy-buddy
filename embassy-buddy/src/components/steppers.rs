use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    peripherals::{
        EXTI1, EXTI2, EXTI5, PA15, PD0, PD1, PD2, PD3, PD4, PD5, PD8, PD9, PD10, PD12, PD13, PD14,
        PD15, PE1, PE2, PE5, USART2,
    },
    usart::{BufferedInterruptHandler, BufferedUart, Config, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, once_lock::OnceLock};
use embassy_tmc::tmc2209::TMC2209AsyncUart;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

static mut STEPPER_TX: [u8; 16] = [0u8; 16];
static mut STEPPER_RX: [u8; 16] = [0u8; 16];
static STEPPER_USART: OnceLock<Mutex<ThreadModeRawMutex, BufferedUart<'static>>> = OnceLock::new();

pub type BuddyStepperInterruptDia<'a> =
    TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'static>>;
pub type BuddyStepperInputDia<'a> =
    TMC2209AsyncUart<'a, Output<'a>, Input<'a>, ThreadModeRawMutex, BufferedUart<'static>>;
pub use embassy_tmc::direction::Direction;

use crate::BoardError;

pub struct BuddySteppers<'a> {
    pub x: BuddyStepperInterruptDia<'a>,
    pub y: BuddyStepperInterruptDia<'a>,
    pub z: BuddyStepperInterruptDia<'a>,
    pub e: BuddyStepperInputDia<'a>,
}

pub(crate) fn init_stepper_usart(usart: USART2, tx: PD5) {
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
        panic!("Stepper USART already initialised");
    };
}

pub(crate) async fn init_x_stepper<'a>(
    en: PD3,
    step: PD1,
    dir: PD0,
    dia_pin: PE2,
    dia_ch: EXTI2,
) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
    if let Some(adc) = STEPPER_USART.try_get() {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        let stepper = TMC2209AsyncUart::new(en, step, dir, dia, 1, adc)?;
        Ok(stepper)
    } else {
        Err(BoardError::Usart2NotInitialised)
    }
}

pub(crate) async fn init_y_stepper<'a>(
    en: PD14,
    step: PD13,
    dir: PD12,
    dia_pin: PE1,
    dia_ch: EXTI1,
) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
    if let Some(adc) = STEPPER_USART.try_get() {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        let stepper = TMC2209AsyncUart::new(en, step, dir, dia, 3, adc)?;
        Ok(stepper)
    } else {
        Err(BoardError::Usart2NotInitialised)
    }
}

pub(crate) async fn init_z_stepper<'a>(
    en: PD2,
    step: PD4,
    dir: PD15,
    dia_pin: PE5,
    dia_ch: EXTI5,
) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
    if let Some(adc) = STEPPER_USART.try_get() {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        let stepper = TMC2209AsyncUart::new(en, step, dir, dia, 0, adc)?;
        Ok(stepper)
    } else {
        Err(BoardError::Usart2NotInitialised)
    }
}

pub(crate) async fn init_e_stepper<'a>(
    en: PD10,
    step: PD9,
    dir: PD8,
    dia_pin: PA15,
) -> Result<BuddyStepperInputDia<'a>, BoardError> {
    if let Some(adc) = STEPPER_USART.try_get() {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = Input::new(dia_pin, Pull::None);
        let stepper = TMC2209AsyncUart::new(en, step, dir, dia, 2, adc)?;
        Ok(stepper)
    } else {
        Err(BoardError::Usart2NotInitialised)
    }
}
