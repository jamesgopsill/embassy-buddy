#![no_std]
#![no_main]

use defmt::*;
use embassy_buddy::Board;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Duration;
use picoserve::{make_static, routing::get};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true; // 16Mhz
        config.rcc.pll_src = PllSource::HSI;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV16,
            mul: PllMul::MUL336,
            divp: Some(PllPDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let stack = Board::init_ethernet(
        &spawner, p.RNG, p.ETH, p.PA1, p.PA2, p.PC1, p.PA7, p.PC4, p.PC5, p.PB12, p.PB13, p.PB11,
        mac_addr,
    )
    .await;

    stack.wait_config_up().await;

    info!("Network task initialized");

    let app = picoserve::Router::new().route("/", get(|| async move { "Hello World" }));

    let port = 3001;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    let config = make_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            persistent_start_read_request: Some(Duration::from_secs(1)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        })
        .keep_connection_alive()
    );

    picoserve::listen_and_serve(
        0,
        &app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}
