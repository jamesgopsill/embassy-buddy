#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    board_config, Board, BuddyFilamentSensor, BuddyPindaSensor, BuddyRotaryButton,
    BuddyRotaryEncoder, BuddyThermistor,
};
use embassy_executor::Spawner;
use embassy_futures::join::{join3, join5};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let mut tx_buf = [0u8; 16];
    let mut rx_buf = [0u8; 16];
    let (owned, shared) = board_config(p, &mut tx_buf, &mut rx_buf);
    let board = Board::new(owned, &shared).await;

    let fut_01 = pinda_interrupt(&board.pinda_sensor);
    let fut_02 = filament_interrupt(&board.filament_sensor);
    let fut_03 = click(&board.rotary_button);
    let fut_04 = spun(&board.rotary_encoder);
    let fut_05 = report_temp(&board.thermistors.bed, "bed");
    let fut_06 = report_temp(&board.thermistors.board, "board");
    let fut_07 = report_temp(&board.thermistors.hotend, "hotend");

    let fut = join5(fut_01, fut_02, fut_03, fut_04, fut_05);
    let fut = join3(fut, fut_06, fut_07);
    fut.await;
}

async fn pinda_interrupt(p: &BuddyPindaSensor<'_>) -> ! {
    let mut p = p.lock().await;
    loop {
        let change = p.on_change().await;
        info!("Pinda: {}", change);
    }
}

async fn filament_interrupt(p: &BuddyFilamentSensor<'_>) -> ! {
    let mut p = p.lock().await;
    loop {
        let change = p.on_change().await;
        info!("Filament: {}", change);
    }
}

async fn click(btn: &BuddyRotaryButton<'_>) -> ! {
    let mut btn = btn.lock().await;
    info!("Initialising Click");
    loop {
        btn.on_click().await;
        info!("click");
    }
}

async fn spun(rot: &BuddyRotaryEncoder<'_>) -> ! {
    info!("Initialising Click");
    let mut rot = rot.lock().await;
    loop {
        let dir = rot.spun().await;
        info!("Spun: {}", dir);
    }
}

async fn report_temp(
    sensor: &BuddyThermistor<'_, '_>,
    label: &str,
) -> ! {
    loop {
        Timer::after_secs(2).await;
        let mut sensor = sensor.lock().await;
        let temp = sensor.read_temperature().await;
        info!("{} Temp: {}", label, temp);
    }
}
