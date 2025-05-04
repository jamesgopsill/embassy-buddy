#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::BuddyMutex;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    peripherals::USART2,
    usart::{BufferedInterruptHandler, BufferedUart, Config, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_tmc::tmc2209::TMC2209AsyncUart;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let en = Output::new(p.PD3, Level::High, Speed::VeryHigh);
    let step = Output::new(p.PD1, Level::High, Speed::VeryHigh);
    let dir = Output::new(p.PD0, Level::High, Speed::VeryHigh);
    let dia = ExtiInput::new(p.PE2, p.EXTI2, Pull::None);

    let mut tx_buf = [0u8; 8];
    let mut rx_buf = [0u8; 16];
    let usart_config = Config::default();
    let usart = BufferedUart::new_half_duplex(
        p.USART2,
        p.PD5,
        Irqs,
        &mut tx_buf,
        &mut rx_buf,
        usart_config,
        HalfDuplexReadback::Readback,
        HalfDuplexConfig::PushPull,
    )
    .unwrap();
    usart.set_baudrate(9_600).unwrap();

    let usart: BuddyMutex<BufferedUart<'_>> = Mutex::new(usart);

    let mut stepper = TMC2209AsyncUart::new(en, step, dir, dia, 1, &usart).unwrap();

    loop {
        let ioin = stepper.read_ioin().await.unwrap();
        info!("Ioin enn: {}", ioin.enn);
        Timer::after_secs(2).await;

        /*
        // Sync + Motor Address + Register + CRC
        let datagram: [u8; 4] = [5, 1, 6, 217];

        info!("Writing: {}", datagram);
        usart.blocking_write(datagram.as_slice()).unwrap();

        info!("Reading");
        let mut buf = [0u8; 8];
        usart.blocking_read(&mut buf).unwrap();
        info!("Read Complete: {}", buf);

        Timer::after_secs(2).await;
        */
    }
}
