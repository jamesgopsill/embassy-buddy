#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;

use embassy_stm32::{
    bind_interrupts,
    peripherals::USART2,
    usart::{BufferedInterruptHandler, BufferedUart, Config, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_time::Timer;
use embedded_io_async::{BufRead, Read, Write};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let mut tx_buf = [0u8; 8];
    let mut rx_buf = [0u8; 16];
    let usart_config = Config::default();
    let mut usart = BufferedUart::new_half_duplex(
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
    info!("USART init");

    // TODO: Receiving bytes back
    // but getting an overrun warning

    loop {
        // usart.flush().await.unwrap();
        let datagram: [u8; 4] = [5, 1, 6, 217];
        let bytes = usart.write(datagram.as_slice()).await.unwrap();
        info!("Bytes Written: {}", bytes);

        info!("Reading Bytes");
        let mut msg = [0u8; 16];
        let n: usize = match datagram.len() {
            4 => 12,
            8 => 16,
            _ => 1,
        };
        //let mut msg = [0u8; 16];
        usart.read_exact(&mut msg[..n]).await.unwrap();
        //let msg = usart.fill_buf().await.unwrap();
        info!("Bytes Received: {}", msg[..n]);
        //usart.consume(msg.len());
        Timer::after_secs(5).await;
    }
}

/* WORKING - now trying buffered uart
let uart_config = Config::default();
let mut usart = Uart::new_blocking_half_duplex(
    p.USART2,
    p.PD5,
    uart_config,
    HalfDuplexReadback::NoReadback,
    HalfDuplexConfig::PushPull,
)
.unwrap();
// usart.set_baudrate(115_200).unwrap();
usart.set_baudrate(9_600).unwrap();
// Sync + Motor Address + Register + CRC
let datagram: [u8; 4] = [5, 1, 6, 217];
info!("Sync: {:#b}", datagram);

Timer::after_micros(1).await;

//info!("Flush");
//usart.blocking_flush().unwrap();

info!("Writing: {}", datagram);

usart.blocking_write(datagram.as_slice()).unwrap();

Timer::after_micros(1).await;

info!("Reading");

let mut buf = [0u8; 8];
usart.blocking_read(&mut buf).unwrap();

Timer::after_micros(1).await;

info!("Read Complete: {}", buf);
*/

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
