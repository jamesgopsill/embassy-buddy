#![allow(clippy::too_many_arguments)]
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_stm32::{
    bind_interrupts,
    eth::{Ethernet, PacketQueue, generic_smi::GenericSMI},
    peripherals::{ETH, PA1, PA2, PA7, PB11, PB12, PB13, PC1, PC4, PC5, RNG},
    rng::Rng,
};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    ETH => embassy_stm32::eth::InterruptHandler;
    RNG => embassy_stm32::rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericSMI>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

pub async fn init_ethernet(
    spawner: &Spawner,
    rng: RNG,
    eth: ETH,
    ref_clk: PA1,
    mdio: PA2,
    mdc: PC1,
    crs: PA7,
    rx_d0: PC4,
    rx_d1: PC5,
    tx_d0: PB12,
    tx_d1: PB13,
    tx_en: PB11,
    mac_addr: [u8; 6],
) -> Stack<'_> {
    // Generate random seed.
    let mut rng = Rng::new(rng, Irqs);
    let mut seed = [0; 8];
    let _ = rng.async_fill_bytes(&mut seed).await;
    let seed = u64::from_le_bytes(seed);

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    // Creating the device according to the Buddy Board pinout.
    //  https://github.com/prusa3d/Buddy-board-MINI-PCB/blob/master/rev.1.0.0/BUDDY_v1.0.0.pdf
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        eth,
        Irqs,
        ref_clk,
        mdio,
        mdc,
        crs,
        rx_d0,
        rx_d1,
        tx_d0,
        tx_d1,
        tx_en,
        GenericSMI::new(0),
        mac_addr,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    spawner.spawn(net_task(runner)).unwrap();

    stack
}
