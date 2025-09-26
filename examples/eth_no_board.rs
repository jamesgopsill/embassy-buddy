#![no_std]
#![no_main]

use core::net::Ipv4Addr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, StackResources};
use embassy_stm32::eth::generic_smi::GenericSMI;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts};
use embassy_time::Duration;
use picoserve::make_static;
use picoserve::routing::get;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ETH => embassy_stm32::eth::InterruptHandler;
    RNG => embassy_stm32::rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericSMI>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Initialising");
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
    info!("Init STM32!");
    let p = embassy_stm32::init(config);
    info!("STM32 initialised");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    info!("RNG initialised");
    let mut seed = [0; 8];
    let _ = rng.async_fill_bytes(&mut seed).await;
    let seed = u64::from_le_bytes(seed);

    info!("Creating the device");

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    // Creating the device according to the Buddy Board pinout.
    //  https://github.com/prusa3d/Buddy-board-MINI-PCB/blob/master/rev.1.0.0/BUDDY_v1.0.0.pdf
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PB12,
        p.PB13,
        p.PB11,
        GenericSMI::new(0),
        mac_addr,
    );

    // let config = embassy_net::Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(192, 168, 0, 250), 24),
        dns_servers: Default::default(),
        gateway: None,
    });

    info!("Initialising the Network Stack");
    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    unwrap!(spawner.spawn(net_task(runner)));

    // info!("Waiting for DHCP");

    // Ensure DHCP configuration is up before trying connect
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
