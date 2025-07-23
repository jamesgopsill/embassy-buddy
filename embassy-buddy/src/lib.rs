#![doc = include_str!("../README.md")]
#![no_std]
#![allow(static_mut_refs, clippy::too_many_arguments)]

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_stm32::{Config, adc::AdcChannel, peripherals::*};
use embassy_tmc::errors::TMCError;
use thiserror::Error;

use crate::components::{
    adc::{self, ADC_SAMPLE_MAX, BUDDY_ADC1},
    bed_power_monitor::{BedPowerMonitor, BuddyPowerMonitor},
    buzzer::{self, BuddyBuzzer},
    eeprom::{self, BuddyEeprom},
    ethernet::init_ethernet,
    fans::{self, BuddyFans},
    filament_sensor::{self, BuddyFilamentSensor},
    flash::{self, BuddyFlash},
    heaters::{self, BuddyHeaters},
    pinda::{self, BuddyPinda},
    rotary_button::{self, BuddyRotaryButton},
    rotary_encoder::{self, BuddyRotaryEncoder},
    steppers::{self, BuddyStepperInputDia, BuddyStepperInterruptDia, BuddySteppers},
    thermistors::{BuddyThermistor, BuddyThermistors, Thermistor},
};

pub mod components;

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
pub struct Board<'a> {
    pub buzzer: BuddyBuzzer<'a>,
    pub bed_power: BuddyPowerMonitor<'a>,
    pub pinda_sensor: BuddyPinda<'a>,
    pub filament_sensor: BuddyFilamentSensor<'a>,
    pub rotary_button: BuddyRotaryButton<'a>,
    pub rotary_encoder: BuddyRotaryEncoder<'a>,
    pub heaters: BuddyHeaters<'a>,
    pub thermistors: BuddyThermistors<'a>,
    pub fans: BuddyFans<'a>,
    pub steppers: BuddySteppers<'a>,
    pub peripherals: BuddyPeripherals,
    pub stack: Stack<'a>,
    pub eeprom: BuddyEeprom<'a>,
    pub flash: BuddyFlash<'a>,
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

impl<'a> Board<'a> {
    pub async fn new(spawner: &'a Spawner, mac_addr: [u8; 6]) -> Self {
        let mut config = Config::default();
        // Configuring STM32 for ethernet
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

        let stack = Self::init_ethernet(
            spawner, p.RNG, p.ETH, p.PA1, p.PA2, p.PC1, p.PA7, p.PC4, p.PC5, p.PB12, p.PB13,
            p.PB11, mac_addr,
        )
        .await;

        // Init statics
        Self::init_adc1(p.ADC1);
        Self::init_stepper_usart(p.USART2, p.PD5);

        // init components
        let buzzer = Self::init_buzzer(p.PA0, p.TIM2);
        let pinda_sensor = Self::init_pinda(p.PA8, p.EXTI8);
        let filament_sensor = Self::init_filament_sensor(p.PB4, p.EXTI4);
        let rotary_button = Self::init_rotary_button(p.PE12, p.EXTI12);
        let rotary_encoder = Self::init_rotary_encoder(p.PE13, p.EXTI13, p.PE15, p.EXTI15);

        // PWM
        let fans = Self::init_fans(p.PE11, p.PE10, p.EXTI10, p.PE9, p.PE14, p.EXTI14, p.TIM1);
        let heaters = Self::init_heaters(p.PB0, p.PB1, p.TIM3);

        // Thermistors (should not fail as we have initialised ADC1).
        let hotend = Self::init_hotend_thermistor(p.PC0, 4_092.0, 100_000.0, 25.0)
            .await
            .unwrap();
        let bed = Self::init_bed_thermistor(p.PA4, 4_550.0, 100_000.0, 25.0)
            .await
            .unwrap();
        let board = Self::init_board_thermistor(p.PA5, 4_267.0, 100_000.0, 25.0)
            .await
            .unwrap();
        let thermistors = BuddyThermistors { hotend, bed, board };

        // Bed Power Monitor (should not fail as we have initialised ADC1).
        let bed_power = Self::init_bed_power_monitor(p.PA3).await.unwrap();

        // Stepper motors (should not fail as we have initialised USART2).
        let x_stepper = Self::init_x_stepper(p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2)
            .await
            .unwrap();
        let y_stepper = Self::init_y_stepper(p.PD14, p.PD13, p.PD12, p.PE1, p.EXTI1)
            .await
            .unwrap();
        let z_stepper = Self::init_z_stepper(p.PD2, p.PD4, p.PD15, p.PE5, p.EXTI5)
            .await
            .unwrap();
        let e_stepper = Self::init_e_stepper(p.PD10, p.PD9, p.PD8, p.PA15)
            .await
            .unwrap();

        let steppers = BuddySteppers {
            x: x_stepper,
            y: y_stepper,
            z: z_stepper,
            e: e_stepper,
        };

        let eeprom = Self::init_eeprom(p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH0);
        let flash = Self::init_flash(
            p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH5, p.DMA1_CH2, p.PD7,
        );

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

        Self {
            buzzer,
            bed_power,
            pinda_sensor,
            filament_sensor,
            rotary_button,
            rotary_encoder,
            heaters,
            thermistors,
            fans,
            steppers,
            peripherals,
            stack,
            eeprom,
            flash,
        }
    }

    pub fn init_adc1(adc: ADC1) {
        adc::init_adc1(adc);
    }

    pub async fn init_ethernet(
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
        init_ethernet(
            spawner, rng, eth, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en, mac_addr,
        )
        .await
    }

    pub async fn init_bed_power_monitor(pin: PA3) -> Result<BuddyPowerMonitor<'a>, BoardError> {
        if let Some(adc) = BUDDY_ADC1.try_get() {
            Ok(BedPowerMonitor::new(
                adc,
                pin.degrade_adc(),
                ADC_SAMPLE_MAX,
                24.0,
            ))
        } else {
            Err(BoardError::Adc1NotInitialised)
        }
    }

    pub fn init_stepper_usart(usart: USART2, tx: PD5) {
        steppers::init_stepper_usart(usart, tx);
    }

    pub fn init_pinda(pin: PA8, ch: EXTI8) -> BuddyPinda<'a> {
        pinda::init_pinda(pin, ch)
    }

    pub fn init_buzzer(ch: PA0, tim: TIM2) -> BuddyBuzzer<'a> {
        buzzer::init_buzzer(ch, tim)
    }

    pub async fn init_default_bed_thermistor(pin: PA4) -> Result<BuddyThermistor<'a>, BoardError> {
        Self::init_bed_thermistor(pin, 4_092.0, 100_000.0, 25.0).await
    }

    pub async fn init_bed_thermistor(
        pin: PA4,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Result<BuddyThermistor<'a>, BoardError> {
        if let Some(adc) = BUDDY_ADC1.try_get() {
            Ok(Thermistor::new(
                adc,
                pin.degrade_adc(),
                4_700.0,
                ADC_SAMPLE_MAX,
                beta,
                r_ref,
                t_ref,
            ))
        } else {
            Err(BoardError::Adc1NotInitialised)
        }
    }

    pub async fn init_default_board_thermistor(
        pin: PA5,
    ) -> Result<BuddyThermistor<'a>, BoardError> {
        Self::init_board_thermistor(pin, 4_550.0, 100_000.0, 25.0).await
    }

    pub async fn init_board_thermistor(
        pin: PA5,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Result<BuddyThermistor<'a>, BoardError> {
        if let Some(adc) = BUDDY_ADC1.try_get() {
            Ok(Thermistor::new(
                adc,
                pin.degrade_adc(),
                4_700.0,
                ADC_SAMPLE_MAX,
                beta,
                r_ref,
                t_ref,
            ))
        } else {
            Err(BoardError::Adc1NotInitialised)
        }
    }

    pub async fn init_default_hotend_thermistor(
        pin: PC0,
    ) -> Result<BuddyThermistor<'a>, BoardError> {
        Self::init_hotend_thermistor(pin, 4_267.0, 100_000.0, 25.0).await
    }

    pub async fn init_hotend_thermistor(
        pin: PC0,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Result<BuddyThermistor<'a>, BoardError> {
        if let Some(adc) = BUDDY_ADC1.try_get() {
            Ok(Thermistor::new(
                adc,
                pin.degrade_adc(),
                4_700.0,
                ADC_SAMPLE_MAX,
                beta,
                r_ref,
                t_ref,
            ))
        } else {
            Err(BoardError::Adc1NotInitialised)
        }
    }

    pub fn init_fans(
        fan_0_pwm: PE11,
        fan_0_inp: PE10,
        fan_0_exti: EXTI10,
        fan_1_pwm: PE9,
        fan_1_inp: PE14,
        fan_1_exti: EXTI14,
        tim: TIM1,
    ) -> BuddyFans<'a> {
        fans::init_fans(
            fan_0_pwm, fan_0_inp, fan_0_exti, fan_1_pwm, fan_1_inp, fan_1_exti, tim,
        )
    }

    pub fn init_filament_sensor(pin: PB4, ch: EXTI4) -> BuddyFilamentSensor<'a> {
        filament_sensor::init_filament_sensor(pin, ch)
    }

    pub fn init_rotary_button(pin: PE12, ch: EXTI12) -> BuddyRotaryButton<'a> {
        rotary_button::init_rotary_button(pin, ch)
    }

    pub fn init_rotary_encoder(
        pin_a: PE13,
        ch_a: EXTI13,
        pin_b: PE15,
        ch_b: EXTI15,
    ) -> BuddyRotaryEncoder<'a> {
        rotary_encoder::init_rotary_encoder(pin_a, ch_a, pin_b, ch_b)
    }

    pub fn init_heaters(bed: PB0, hotend: PB1, tim: TIM3) -> BuddyHeaters<'a> {
        heaters::init_heaters(bed, hotend, tim)
    }

    pub async fn init_x_stepper(
        en: PD3,
        step: PD1,
        dir: PD0,
        dia_pin: PE2,
        dia_ch: EXTI2,
    ) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
        steppers::init_x_stepper(en, step, dir, dia_pin, dia_ch).await
    }

    pub async fn init_y_stepper(
        en: PD14,
        step: PD13,
        dir: PD12,
        dia_pin: PE1,
        dia_ch: EXTI1,
    ) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
        steppers::init_y_stepper(en, step, dir, dia_pin, dia_ch).await
    }

    pub async fn init_z_stepper(
        en: PD2,
        step: PD4,
        dir: PD15,
        dia_pin: PE5,
        dia_ch: EXTI5,
    ) -> Result<BuddyStepperInterruptDia<'a>, BoardError> {
        steppers::init_z_stepper(en, step, dir, dia_pin, dia_ch).await
    }

    pub async fn init_e_stepper(
        en: PD10,
        step: PD9,
        dir: PD8,
        dia_pin: PA15,
    ) -> Result<BuddyStepperInputDia<'a>, BoardError> {
        steppers::init_e_stepper(en, step, dir, dia_pin).await
    }

    pub fn init_eeprom(
        peri: I2C1,
        scl: PB8,
        sda: PB9,
        tx_dma: DMA1_CH6,
        rx_dma: DMA1_CH0,
    ) -> BuddyEeprom<'a> {
        eeprom::init_eeprom(peri, scl, sda, tx_dma, rx_dma)
    }

    pub fn init_flash(
        peri: SPI3,
        sck: PC10,
        mosi: PC12,
        miso: PC11,
        tx_dma: DMA1_CH5,
        rx_dma: DMA1_CH2,
        cs: PD7,
    ) -> BuddyFlash<'a> {
        flash::init_flash(peri, sck, mosi, miso, tx_dma, rx_dma, cs)
    }
}
