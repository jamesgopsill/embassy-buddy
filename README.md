# embassy-buddy

A Board Support Crate (BSP) for the Prusa Buddy board that powers the Prusa Mini. The crate is **still in development**.

# Example

Examples can be found in the `examples` folder on GitHub. These can be run using:

```
cargo run --example buzzer
```

And here is the `examples/buzzer.rs`.

```Rust
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{buzzer::Buzzer, Board, BuddyMutex};
use embassy_executor::Spawner;
use embassy_stm32::peripherals::TIM2;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(buzz(&board.buzzer));
}

#[embassy_executor::task()]
pub async fn buzz(buzzer: &'static BuddyMutex<Buzzer<TIM2>>) {
	let mut guard = buzzer.lock().await;
	let buzzer = guard.as_mut().unwrap();
	buzzer.enable().await;
	let mut n = 0;
	loop {
		info!("Buzz");
		buzzer.set_duty_cycle_fraction(1, 2).await;
		Timer::after_millis(100).await;
		buzzer.set_duty_cycle_fully_off().await;
		Timer::after_secs(1).await;
		n += 1;
		if n > 5 {
			break;
		}
	}
	info!("Buzz Finished");
}
```

# Support

Please consider supporting the crate by:

- Downloading and using the crate.
- Raising issues and improvements on GitHub.
- Recommending the crate to others.
- ⭐ the crate on GitHub.
- Sponsoring the [maintainer](https://github.com/sponsors/jamesgopsill).

# Who's using it

Well me for starters. My research is developing and studying decentalised trusted Artificial Intelligent (AI) agent networks. This crate is contributing to a reference model demonstrating how we can embed AIgents in manufacturing machines. The machines then join networks where they can broker work with one another an AIgents who are acting on behalf of jobs in the network. We have a network set up in our lab that handles our own job loads.

The trust element is required in order to prove that the AIgents are who they say they are. For example, the machine I am talking to is the machine I think it is and a job is coming from someone I know it is. We achieve this by using blockchain, self-soverign identities, verifiable credentials, smart contract, Hardware Unique Key (HUK) and One-Time Programming (OTP) technologies.

We have selected Rust as the language for the entire reference model stack oweing to its memory-safety, performance, and we can use it for embedded systems, desktop, webapp, and server-side applications.

# Contribute

Sure, send me a message.

# Shout-Outs and References

- Prusa and the opensource/hardware software reposoitories
	- [Prusa Buddy Firmware](https://github.com/prusa3d/Prusa-Firmware-Buddy)
	- [Prusa Buddy Board Documentation](https://github.com/prusa3d/Buddy-board-MINI-PCB)
- The [Embassy](https://embassy.dev/) crate and all its examples.
- Youtubers showing me how to Rust.
	- [Jon Gjengset](https://www.youtube.com/@jonhoo)
	- [The Rusty Bits](https://www.youtube.com/@therustybits)
	- [Floodplain](https://www.youtube.com/@floodplainnl)
- [TM2209 Datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.08.pdf)
- [Embedded Rustacean Blog](https://www.theembeddedrustacean.com/)
