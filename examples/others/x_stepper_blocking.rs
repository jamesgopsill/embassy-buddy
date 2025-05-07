#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;

use embassy_stm32::usart::{Config, HalfDuplexConfig, HalfDuplexReadback, Uart};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    // WORKING - now trying buffered uart
    let uart_config = Config::default();
    let mut usart = Uart::new_blocking_half_duplex(
        p.USART2,
        p.PD5,
        uart_config,
        HalfDuplexReadback::NoReadback,
        HalfDuplexConfig::PushPull,
    )
    .unwrap();
    // setting a low baud because it doesn't need
    // to be any faster for now and trying 115_200 had
    // issues.
    usart.set_baudrate(9_600).unwrap();

    loop {
        // Sync + Motor Address + Register + CRC
        let datagram: [u8; 4] = [5, 1, 6, 217];

        info!("Writing: {}", datagram);
        usart.blocking_write(datagram.as_slice()).unwrap();

        info!("Reading");
        let mut buf = [0u8; 8];
        usart.blocking_read(&mut buf).unwrap();
        info!("Read Complete: {}", buf);

        Timer::after_secs(2).await;
    }
}

/*

let usart = Mutex::new(usart);

let mut stepper = Board::init_x_stepper(p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2, &usart);

info!("Attempting usart connection stepper");
let ioin = stepper.read_ioin().await.unwrap();
info!("Enabled: {}", ioin.enn);

stepper.enable();
info!("Stepper Enabled");
let mut n = 0;
let microstep = 64;
loop {
    n += 1;
    if n > 2 {
        break;
    }
    info!("Forward");
    stepper.set_direction(Direction::Clockwise);
    for _ in 0..100 * microstep {
        stepper.step();
        Timer::after_ticks(1).await
    }
    info!("Backward");
    stepper.set_direction(Direction::CounterClockwise);
    for _ in 0..100 * microstep {
        stepper.step();
        Timer::after_ticks(1).await
    }
}
info!("Stepper Disabled");
stepper.disable();
*/
