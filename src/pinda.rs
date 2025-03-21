use defmt::Format;
use embassy_stm32::{exti::AnyChannel, exti::ExtiInput, gpio::AnyPin};

pub struct Pinda {
    input: ExtiInput<'static>,
}

#[derive(Debug, Format)]
pub enum PindaStateChange {
    Contact,
    NoContact,
}

impl Pinda {
    pub async fn init(pin: AnyPin, exti: AnyChannel) -> Pinda {
        let input = ExtiInput::new(pin, exti, embassy_stm32::gpio::Pull::None);
        Pinda { input }
    }

    pub fn in_contact(&mut self) {
        self.input.is_high();
    }

    pub async fn wait_for_contact(&mut self) {
        self.input.wait_for_rising_edge().await;
    }

    pub async fn wait_for_no_contact(&mut self) {
        self.input.wait_for_falling_edge().await;
    }

    pub async fn on_change(&mut self) -> PindaStateChange {
        self.input.wait_for_any_edge().await;
        if self.input.is_high() {
            PindaStateChange::Contact
        } else {
            PindaStateChange::NoContact
        }
    }
}
