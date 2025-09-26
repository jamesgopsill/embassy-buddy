#![doc = include_str!("../../docs/steppers.md")]
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    peripherals::{
        EXTI1, EXTI2, EXTI5, PA15, PD0, PD1, PD2, PD3, PD4, PD5, PD8, PD9, PD10, PD12, PD13, PD14,
        PD15, PE1, PE2, PE5, USART2,
    },
    usart::{BufferedInterruptHandler, BufferedUart, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use static_cell::StaticCell;

use crate::TMC2209;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

pub type BuddyStepperExti<'a> =
    TMC2209<'a, ThreadModeRawMutex, Output<'a>, ExtiInput<'a>, BufferedUart<'static>>;
pub type BuddyStepperInp<'a> =
    TMC2209<'a, ThreadModeRawMutex, Output<'a>, Input<'a>, BufferedUart<'static>>;

pub type BuddyUart = Mutex<ThreadModeRawMutex, BufferedUart<'static>>;

pub(crate) fn build_stepper_usart(usart: USART2, tx: PD5) -> &'static BuddyUart {
    // 20 is the maximum expected from a write request.
    static TX: StaticCell<[u8; 20]> = StaticCell::new();
    let tx_buf = TX.init([0u8; 20]);
    // Only tx is used as it is a single wire. Can have a minmal RX buf.
    static RX: StaticCell<[u8; 1]> = StaticCell::new();
    let rx_buf = RX.init([0u8; 1]);
    static UART: StaticCell<BuddyUart> = StaticCell::new();
    let uart = BufferedUart::new_half_duplex(
        usart,
        tx,
        Irqs,
        tx_buf,
        rx_buf,
        Default::default(),
        // Must be set to Readback for the uart to work.
        HalfDuplexReadback::Readback,
        HalfDuplexConfig::PushPull,
    )
    .unwrap();
    UART.init(Mutex::new(uart))
}

pub(crate) fn build_x_stepper<'a>(
    usart: &'a BuddyUart,
    en: PD3,
    step: PD1,
    dir: PD0,
    dia_pin: PE2,
    dia_ch: EXTI2,
) -> BuddyStepperExti<'a> {
    let en = Output::new(en, Level::High, Speed::VeryHigh);
    let step = Output::new(step, Level::High, Speed::VeryHigh);
    let dir = Output::new(dir, Level::High, Speed::VeryHigh);
    let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
    TMC2209::new_async_usart_interruptable(en, step, dir, dia, 1, usart).unwrap()
}

pub(crate) fn build_y_stepper<'a>(
    usart: &'a BuddyUart,
    en: PD14,
    step: PD13,
    dir: PD12,
    dia_pin: PE1,
    dia_ch: EXTI1,
) -> BuddyStepperExti<'a> {
    let en = Output::new(en, Level::High, Speed::VeryHigh);
    let step = Output::new(step, Level::High, Speed::VeryHigh);
    let dir = Output::new(dir, Level::High, Speed::VeryHigh);
    let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
    TMC2209::new_async_usart_interruptable(en, step, dir, dia, 3, usart).unwrap()
}

pub(crate) fn build_z_stepper<'a>(
    usart: &'a BuddyUart,
    en: PD2,
    step: PD4,
    dir: PD15,
    dia_pin: PE5,
    dia_ch: EXTI5,
) -> BuddyStepperExti<'a> {
    let en = Output::new(en, Level::High, Speed::VeryHigh);
    let step = Output::new(step, Level::High, Speed::VeryHigh);
    let dir = Output::new(dir, Level::High, Speed::VeryHigh);
    let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
    TMC2209::new_async_usart_interruptable(en, step, dir, dia, 0, usart).unwrap()
}

pub(crate) fn build_e_stepper<'a>(
    usart: &'a BuddyUart,
    en: PD10,
    step: PD9,
    dir: PD8,
    dia_pin: PA15,
) -> BuddyStepperInp<'a> {
    let en = Output::new(en, Level::High, Speed::VeryHigh);
    let step = Output::new(step, Level::High, Speed::VeryHigh);
    let dir = Output::new(dir, Level::High, Speed::VeryHigh);
    let dia = Input::new(dia_pin, Pull::None);
    TMC2209::new_async_usart_no_interrupt(en, step, dir, dia, 2, usart).unwrap()
}
