#![doc = include_str!("../README.md")]
#![no_std]
#![allow(static_mut_refs, clippy::too_many_arguments)]

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_stm32::{
    Config,
    adc::AdcChannel,
    exti::ExtiInput,
    gpio::{OutputType, Pull},
    peripherals::*,
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use thiserror::Error;

use crate::components::{
    adc::BuddyAdc, bed_power_monitor, buzzer, display, eeprom, ethernet::build_ethernet, fans::Fan,
    filament_sensor, flash, heaters::Heater, pinda, rotary_button, rotary_encoder, steppers,
    thermistors::Thermistor,
};

pub mod components;
pub(crate) mod fmt;
use crate::fmt::info;

pub use crate::components::bed_power_monitor::BuddyBedPowerMonitor;
pub use crate::components::buzzer::BuddyBuzzer;
pub use crate::components::display::BuddyDisplay;
pub use crate::components::eeprom::BuddyEeprom;
pub use crate::components::fans::BuddyFan;
pub use crate::components::filament_sensor::BuddyFilamentSensor;
pub use crate::components::flash::BuddyFlash;
pub use crate::components::heaters::BuddyHeater;
pub use crate::components::pinda::BuddyPinda;
pub use crate::components::rotary_button::BuddyRotaryButton;
pub use crate::components::rotary_encoder::BuddyRotaryEncoder;
pub use crate::components::steppers::{BuddyStepperExti, BuddyStepperInp};
pub use crate::components::thermistors::BuddyThermistor;
pub use crate::components::tmc::*;

pub use crate::components::tmc::{
    ChopConf, GStat, Gconf, IHoldIRun, IfCnt, Ioin, NodeConf, PwmConf, TCoolThrs, TMCError,
    TPowerDown, TStep, TpwmThrs, VActual,
};

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("ADC1 has not been initialised.")]
    Adc1NotInitialised,
    #[error("USART2 has not been initialised.")]
    Usart2NotInitialised,
    #[error("TMCError.")]
    TMCError(#[from] TMCError),
}

/// The buddy board struct.
#[derive(Default)]
pub struct Board<'a> {
    pub bed_power: Option<BuddyBedPowerMonitor>,
    pub buzzer: Option<BuddyBuzzer<'a>>,
    pub stack: Option<Stack<'a>>,
    pub pinda: Option<BuddyPinda<'a>>,
    pub filament_sensor: Option<BuddyFilamentSensor<'a>>,
    pub rotary_button: Option<BuddyRotaryButton<'a>>,
    pub rotary_encoder: Option<BuddyRotaryEncoder<'a>>,
    pub bed_heater: Option<BuddyHeater<'a>>,
    pub hotend_heater: Option<BuddyHeater<'a>>,
    pub hotend_thermistor: Option<BuddyThermistor<'a>>,
    pub bed_thermistor: Option<BuddyThermistor<'a>>,
    pub board_thermistor: Option<BuddyThermistor<'a>>,
    pub fan_0: Option<BuddyFan<'a>>,
    pub fan_1: Option<BuddyFan<'a>>,
    pub x_stepper: Option<BuddyStepperExti<'a>>,
    pub y_stepper: Option<BuddyStepperExti<'a>>,
    pub z_stepper: Option<BuddyStepperExti<'a>>,
    pub e_stepper: Option<BuddyStepperInp<'a>>,
    pub peripherals: Option<BuddyPeripherals>,
    pub eeprom: Option<BuddyEeprom<'a>>,
    pub flash: Option<BuddyFlash<'a>>,
    pub display: Option<BuddyDisplay<'a>>,
}

/// The peripherals that are not used by default but are, for example, available through expansion headers on the board.
pub struct BuddyPeripherals {
    pub adc2: ADC2,
    pub adc3: ADC3,
    // J10 (ESP) header
    pub j10: J10,
    // Conn_01x10 header
    pub conn01x10: Conn01x10,
    // USARTS
    pub usart1: USART1,
    // Timers
    pub tim5: TIM5,
    pub tim6: TIM6,
    pub tim7: TIM7,
    pub tim8: TIM8,
    pub tim9: TIM9,
    pub tim10: TIM10,
    pub tim11: TIM11,
    pub tim12: TIM12,
    pub tim13: TIM13,
    pub tim14: TIM14,
}

pub struct J10 {
    pub pe6: PE6,
    pub pc6: PC6,
    pub pc7: PC7,
    pub pc13: PC13,
}

pub struct Conn01x10 {
    pub pe0: PE0,
    pub pb5: PB5,
    pub pb6: PB6,
    pub pb7: PB7,
    // TODO: I2C pass-through.
    //pub pb8: PB8,
    //pub pb9: PB9,
}

#[derive(Default)]
pub struct BoardBuilder<'a> {
    bed_power: bool,
    buzzer: bool,
    ethernet: bool,
    filament_sensor: bool,
    pinda: bool,
    rotary_button: bool,
    rotary_encoder: bool,
    bed_heater: bool,
    hotend_heater: bool,
    bed_thermistor: bool,
    bed_beta: f64,
    bed_r_ref: f64,
    bed_t_ref: f64,
    board_thermistor: bool,
    board_beta: f64,
    board_r_ref: f64,
    board_t_ref: f64,
    hotend_thermistor: bool,
    hotend_beta: f64,
    hotend_r_ref: f64,
    hotend_t_ref: f64,
    fan_0: bool,
    fan_1: bool,
    x_stepper: bool,
    y_stepper: bool,
    z_stepper: bool,
    e_stepper: bool,
    eeprom: bool,
    flash: bool,
    display: bool,
    mac_addr: [u8; 6],
    spawner: Option<&'a Spawner>,
}

impl<'a> BoardBuilder<'a> {
    pub fn new() -> BoardBuilder<'a> {
        Self::default()
            .bed_heater(true)
            .bed_thermistor(true, None, None, None)
            .bed_power(true)
            .board_thermistor(true, None, None, None)
            .buzzer(true)
            .display(true)
            .eeprom(true)
            .e_stepper(true)
            .fan_0(true)
            .fan_1(true)
            .filament_sensor(true)
            .flash(true)
            .hotend_thermistor(true, None, None, None)
            .pinda(true)
            .rotary_button(true)
            .rotary_encoder(true)
            .x_stepper(true)
            .y_stepper(true)
            .z_stepper(true)
    }

    pub fn buzzer(mut self, build: bool) -> BoardBuilder<'a> {
        self.buzzer = build;
        self
    }

    fn build_buzzer(ch: PA0, tim: TIM2) -> BuddyBuzzer<'a> {
        buzzer::build_buzzer(ch, tim)
    }

    pub fn ethernet(mut self, spawner: &'a Spawner, mac_addr: [u8; 6]) -> BoardBuilder<'a> {
        self.ethernet = true;
        self.spawner = Some(spawner);
        self.mac_addr = mac_addr;
        self
    }

    async fn build_ethernet(
        spawner: &'a Spawner,
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
    ) -> Stack<'a> {
        build_ethernet(
            spawner, rng, eth, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en, mac_addr,
        )
        .await
    }

    pub fn bed_power(mut self, build: bool) -> BoardBuilder<'a> {
        self.bed_power = build;
        self
    }

    fn build_bed_power_monitor(pin: PA3, adc: &'static BuddyAdc<ADC1>) -> BuddyBedPowerMonitor {
        bed_power_monitor::build_buddy_bed_power_monitor(adc, pin.degrade_adc())
    }

    pub fn pinda(mut self, build: bool) -> BoardBuilder<'a> {
        self.pinda = build;
        self
    }

    fn build_pinda(pin: PA8, ch: EXTI8) -> BuddyPinda<'a> {
        pinda::init_pinda(pin, ch)
    }

    pub fn filament_sensor(mut self, build: bool) -> BoardBuilder<'a> {
        self.filament_sensor = build;
        self
    }

    fn build_filament_sensor(pin: PB4, ch: EXTI4) -> BuddyFilamentSensor<'a> {
        filament_sensor::build_filament_sensor(pin, ch)
    }

    pub fn rotary_button(mut self, build: bool) -> BoardBuilder<'a> {
        self.rotary_button = build;
        self
    }

    pub fn rotary_encoder(mut self, build: bool) -> BoardBuilder<'a> {
        self.rotary_encoder = build;
        self
    }

    fn build_rotary_button(pin: PE12, ch: EXTI12) -> BuddyRotaryButton<'a> {
        rotary_button::build_rotary_button(pin, ch)
    }

    fn build_rotary_encoder(
        pin_a: PE13,
        ch_a: EXTI13,
        pin_b: PE15,
        ch_b: EXTI15,
    ) -> BuddyRotaryEncoder<'a> {
        rotary_encoder::build_rotary_encoder(pin_a, ch_a, pin_b, ch_b)
    }

    pub fn bed_heater(mut self, build: bool) -> BoardBuilder<'a> {
        self.bed_heater = build;
        self
    }

    pub fn bed_thermistor(
        mut self,
        build: bool,
        beta: Option<f64>,
        r_ref: Option<f64>,
        t_tref: Option<f64>,
    ) -> BoardBuilder<'a> {
        self.bed_thermistor = build;
        if self.bed_thermistor {
            self.bed_beta = beta.unwrap_or(4_092.0);
            self.bed_r_ref = r_ref.unwrap_or(100_000.0);
            self.bed_t_ref = t_tref.unwrap_or(25.0);
        }
        self
    }

    pub fn board_thermistor(
        mut self,
        build: bool,
        beta: Option<f64>,
        r_ref: Option<f64>,
        t_tref: Option<f64>,
    ) -> BoardBuilder<'a> {
        self.board_thermistor = build;
        if self.board_thermistor {
            self.board_beta = beta.unwrap_or(4_550.0);
            self.board_r_ref = r_ref.unwrap_or(100_000.0);
            self.board_t_ref = t_tref.unwrap_or(25.0);
        }
        self
    }

    pub fn hotend_thermistor(
        mut self,
        build: bool,
        beta: Option<f64>,
        r_ref: Option<f64>,
        t_tref: Option<f64>,
    ) -> BoardBuilder<'a> {
        self.hotend_thermistor = build;
        if self.hotend_thermistor {
            self.hotend_beta = beta.unwrap_or(4_267.0);
            self.hotend_r_ref = r_ref.unwrap_or(100_000.0);
            self.hotend_t_ref = t_tref.unwrap_or(25.0);
        }
        self
    }

    pub fn fan_0(mut self, build: bool) -> BoardBuilder<'a> {
        self.fan_0 = build;
        self
    }

    pub fn fan_1(mut self, build: bool) -> BoardBuilder<'a> {
        self.fan_1 = build;
        self
    }

    pub fn x_stepper(mut self, build: bool) -> BoardBuilder<'a> {
        self.x_stepper = build;
        self
    }

    pub fn y_stepper(mut self, build: bool) -> BoardBuilder<'a> {
        self.y_stepper = build;
        self
    }

    pub fn z_stepper(mut self, build: bool) -> BoardBuilder<'a> {
        self.z_stepper = build;
        self
    }

    pub fn e_stepper(mut self, build: bool) -> BoardBuilder<'a> {
        self.e_stepper = build;
        self
    }

    pub fn eeprom(mut self, build: bool) -> BoardBuilder<'a> {
        self.eeprom = build;
        self
    }

    pub fn flash(mut self, build: bool) -> BoardBuilder<'a> {
        self.flash = build;
        self
    }

    pub fn display(mut self, build: bool) -> BoardBuilder<'a> {
        self.display = build;
        self
    }

    fn build_adc1(adc: ADC1) -> &'static BuddyAdc<ADC1> {
        BuddyAdc::new_static_adc1(adc)
    }

    pub async fn build(self) -> Board<'a> {
        let mut board = Board::default();
        let mut config = Config::default();
        if self.ethernet {
            info!("[BUDDY] Building Ethernet");
            // Configuring STM32 for ethernet
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
        if self.ethernet {
            let stack = Self::build_ethernet(
                self.spawner.unwrap(),
                p.RNG,
                p.ETH,
                p.PA1,
                p.PA2,
                p.PC1,
                p.PA7,
                p.PC4,
                p.PC5,
                p.PB12,
                p.PB13,
                p.PB11,
                self.mac_addr,
            )
            .await;
            board.stack = Some(stack)
        }
        if self.buzzer {
            info!("[BUDDY] Building Buzzer");
            let buzzer = Self::build_buzzer(p.PA0, p.TIM2);
            board.buzzer = Some(buzzer);
        }
        if self.bed_power || self.bed_thermistor || self.board_thermistor || self.hotend_thermistor
        {
            let adc = Self::build_adc1(p.ADC1);
            if self.bed_power {
                info!("[BUDDY] Building Bed Power Monitor");
                let bed_power = Self::build_bed_power_monitor(p.PA3, adc);
                board.bed_power = Some(bed_power);
            }
            if self.bed_thermistor {
                info!("[BUDDY] Building Bed Thermistor");
                let thermistor = Thermistor::new(
                    adc,
                    p.PA4.degrade_adc(),
                    4_700.0,
                    self.bed_beta,
                    self.bed_r_ref,
                    self.bed_t_ref,
                );
                board.bed_thermistor = Some(thermistor);
            }
            if self.board_thermistor {
                info!("[BUDDY] Building Board Thermistor");
                let thermistor = Thermistor::new(
                    adc,
                    p.PA5.degrade_adc(),
                    4_700.0,
                    self.board_beta,
                    self.board_r_ref,
                    self.board_t_ref,
                );
                board.board_thermistor = Some(thermistor);
            }
            if self.hotend_thermistor {
                info!("[BUDDY] Building Hotend Thermistor");
                let thermistor = Thermistor::new(
                    adc,
                    p.PC0.degrade_adc(),
                    4_700.0,
                    self.hotend_beta,
                    self.hotend_r_ref,
                    self.hotend_t_ref,
                );
                board.hotend_thermistor = Some(thermistor);
            }
        }
        if self.pinda {
            info!("[BUDDY] Building Pinda");
            let pinda = Self::build_pinda(p.PA8, p.EXTI8);
            board.pinda = Some(pinda);
        }
        if self.filament_sensor {
            info!("[BUDDY] Building Filament Sensor");
            let filament_sensor = Self::build_filament_sensor(p.PB4, p.EXTI4);
            board.filament_sensor = Some(filament_sensor);
        }
        if self.rotary_button {
            info!("[BUDDY] Building Rotary Button");
            let rotary_button = Self::build_rotary_button(p.PE12, p.EXTI12);
            board.rotary_button = Some(rotary_button);
        }
        if self.rotary_encoder {
            info!("[BUDDY] Building Rotary Encoder");
            let rotary_encoder = Self::build_rotary_encoder(p.PE13, p.EXTI13, p.PE15, p.EXTI15);
            board.rotary_encoder = Some(rotary_encoder);
        }
        if self.bed_heater || self.hotend_heater {
            let bed_pin = if self.bed_heater {
                Some(PwmPin::new_ch3(p.PB0, OutputType::PushPull))
            } else {
                None
            };

            let hotend_pin = if self.bed_heater {
                Some(PwmPin::new_ch4(p.PB1, OutputType::PushPull))
            } else {
                None
            };

            let pwm = SimplePwm::new(
                p.TIM3,
                None,
                None,
                bed_pin,
                hotend_pin,
                khz(21),
                Default::default(),
            );
            let channels = pwm.split();
            // TODO: enable the channels
            if self.bed_heater {
                info!("[BUDDY] Building Bed Heater");
                let heater = Heater::new(channels.ch3);
                board.bed_heater = Some(heater);
            }
            if self.hotend_heater {
                info!("[BUDDY] Building Hotend Heater");
                let heater = Heater::new(channels.ch4);
                board.hotend_heater = Some(heater);
            }
        }
        if self.fan_0 || self.fan_1 {
            let fan_0_pwm_pin = if self.fan_0 {
                info!("[BUDDY] Building Fan 0");
                Some(PwmPin::new_ch2(p.PE11, OutputType::PushPull))
            } else {
                None
            };

            let fan_1_pwm_pin = if self.fan_1 {
                info!("[BUDDY] Building Fan 1");
                Some(PwmPin::new_ch1(p.PE9, OutputType::PushPull))
            } else {
                None
            };

            let pwm = SimplePwm::new(
                p.TIM1,
                fan_1_pwm_pin,
                fan_0_pwm_pin,
                None,
                None,
                khz(21),
                Default::default(),
            );

            let channels = pwm.split();

            if self.fan_0 {
                let mut ch = channels.ch2;
                ch.enable();
                let exti = ExtiInput::new(p.PE10, p.EXTI10, Pull::Down);
                let fan = Fan::new(ch, exti);
                board.fan_0 = Some(fan);
            }

            if self.fan_1 {
                let mut ch = channels.ch1;
                ch.enable();
                let exti = ExtiInput::new(p.PE14, p.EXTI14, Pull::Down);
                let fan = Fan::new(ch, exti);
                board.fan_1 = Some(fan);
            }
        }

        if self.x_stepper || self.y_stepper || self.z_stepper || self.e_stepper {
            let usart = steppers::build_stepper_usart(p.USART2, p.PD5);
            if self.x_stepper {
                info!("[BUDDY] Building Stepper X");
                let stepper = steppers::build_x_stepper(usart, p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2);
                board.x_stepper = Some(stepper);
            }
            if self.y_stepper {
                info!("[BUDDY] Building Stepper Y");
                let stepper =
                    steppers::build_y_stepper(usart, p.PD14, p.PD13, p.PD12, p.PE1, p.EXTI1);
                board.y_stepper = Some(stepper);
            }
            if self.z_stepper {
                info!("[BUDDY] Building Stepper Z");
                let stepper =
                    steppers::build_z_stepper(usart, p.PD2, p.PD4, p.PD15, p.PE5, p.EXTI5);
                board.z_stepper = Some(stepper);
            }
            if self.e_stepper {
                info!("[BUDDY] Building Stepper E");
                let stepper = steppers::build_e_stepper(usart, p.PD10, p.PD9, p.PD8, p.PA15);
                board.e_stepper = Some(stepper);
            }
        }

        if self.eeprom {
            info!("[BUDDY] Building EEPROM");
            let eeprom = eeprom::build_eeprom(p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH5);
            board.eeprom = Some(eeprom);
        }

        if self.flash {
            info!("[BUDDY] Building FLASH");
            let flash = flash::build_flash(
                p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH7, p.DMA1_CH0, p.PD7,
            );
            board.flash = Some(flash);
        }

        if self.display {
            info!("[BUDDY] Building Display");
            let display =
                display::build_display(p.SPI2, p.PB10, p.PC3, p.PC2, p.PC9, p.PD11, p.PC8);
            board.display = Some(display);
        }

        let j10 = J10 {
            pe6: p.PE6,
            pc6: p.PC6,
            pc7: p.PC7,
            pc13: p.PC13,
        };

        let conn01x10 = Conn01x10 {
            pe0: p.PE0,
            pb5: p.PB5,
            pb6: p.PB6,
            pb7: p.PB7,
            //pb8: p.PB8,
            //pb9: p.PB9,
        };

        let peripherals = BuddyPeripherals {
            adc2: p.ADC2,
            adc3: p.ADC3,
            j10,
            conn01x10,
            usart1: p.USART1,
            tim5: p.TIM5,
            tim6: p.TIM6,
            tim7: p.TIM7,
            tim8: p.TIM8,
            tim9: p.TIM9,
            tim10: p.TIM10,
            tim11: p.TIM11,
            tim12: p.TIM12,
            tim13: p.TIM13,
            tim14: p.TIM14,
        };

        board.peripherals = Some(peripherals);

        board
    }
}
