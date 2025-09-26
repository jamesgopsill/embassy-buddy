#![no_std]
#![no_main]

use defmt::*;
use embassy_buddy::BoardBuilder;
use embassy_executor::Spawner;
use embassy_time::Duration;
use picoserve::{make_static, routing::get};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let board = BoardBuilder::default()
        .ethernet(&spawner, mac_addr)
        .build()
        .await;
    let stack = board.stack.unwrap();

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
