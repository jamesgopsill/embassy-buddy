#![doc = include_str!("../README.md")]
#![no_std]

use components::{
    buzzer::Buzzer, fan::Fan, filament_sensor::FilamentSensor, heater::Heater, pinda::Pinda,
    rotary_button::RotaryButton, rotary_encoder::RotaryEncoder, thermistor::Thermistor,
};
use defmt::info;
use embassy_stm32::{
    Peripherals,
    adc::{Adc, AdcChannel},
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{
        ADC1, EXTI1, EXTI2, EXTI4, EXTI5, EXTI8, EXTI10, EXTI12, EXTI13, EXTI14, EXTI15, PA0, PA4,
        PA5, PA8, PA15, PB0, PB1, PB4, PC0, PD0, PD1, PD2, PD3, PD4, PD8, PD9, PD10, PD12, PD13,
        PD14, PD15, PE1, PE2, PE5, PE9, PE10, PE11, PE12, PE13, PE14, PE15, TIM1, TIM2, TIM3,
        USART2,
    },
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
    usart::{BufferedInterruptHandler, BufferedUart, Config, HalfDuplexConfig, HalfDuplexReadback},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_tmc::{errors::TMCError, tmc2209::TMC2209AsyncUart};

pub mod components;

pub type BuddyMutex<T> = Mutex<ThreadModeRawMutex, T>;
pub type BuddyPindaSensor<'a> = BuddyMutex<Pinda<ExtiInput<'a>>>;
pub type BuddyFilamentSensor<'a> = BuddyMutex<FilamentSensor<ExtiInput<'a>>>;
pub type BuddyRotaryButton<'a> = BuddyMutex<RotaryButton<ExtiInput<'a>>>;
pub type BuddyThermistor<'a, 'b> = BuddyMutex<Thermistor<'a, 'b>>;
pub type BuddyRotaryEncoder<'a> = BuddyMutex<RotaryEncoder<ExtiInput<'a>>>;
pub type BuddyBuzzer<'a> = BuddyMutex<Buzzer<SimplePwmChannel<'a, TIM2>>>;
pub type BuddyStepperInterrupt<'a, 'b> = BuddyMutex<
    TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'b>>,
>;
pub type BuddyStepperInput<'a, 'b> =
    BuddyMutex<TMC2209AsyncUart<'a, Output<'a>, Input<'a>, ThreadModeRawMutex, BufferedUart<'b>>>;
pub type BuddyFan<'a> = BuddyMutex<Fan<SimplePwmChannel<'a, TIM1>, ExtiInput<'a>>>;
pub type BuddyHeater<'a> = BuddyMutex<Heater<SimplePwmChannel<'a, TIM3>>>;

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<USART2>;
});

#[derive(Debug, Default)]
pub struct BoardBuffers {
    pub stepper_tx: [u8; 16],
    pub stepper_rx: [u8; 16],
}

pub struct BoardOwned {
    pub pa0: PA0,
    pub pa4: PA4,
    pub pa5: PA5,
    pub pa8: PA8,
    pub pa15: PA15,
    pub pb0: PB0,
    pub pb1: PB1,
    pub pb4: PB4,
    pub pc0: PC0,
    pub pd0: PD0,
    pub pd1: PD1,
    pub pd2: PD2,
    pub pd4: PD4,
    pub pd3: PD3,
    pub pd8: PD8,
    pub pd9: PD9,
    pub pd10: PD10,
    pub pd12: PD12,
    pub pd13: PD13,
    pub pd14: PD14,
    pub pd15: PD15,
    pub pe1: PE1,
    pub pe2: PE2,
    pub pe5: PE5,
    pub pe9: PE9,
    pub pe10: PE10,
    pub pe11: PE11,
    pub pe12: PE12,
    pub pe13: PE13,
    pub pe14: PE14,
    pub pe15: PE15,
    pub exti1: EXTI1,
    pub exti2: EXTI2,
    pub exti4: EXTI4,
    pub exti5: EXTI5,
    pub exti8: EXTI8,
    pub exti10: EXTI10,
    pub exti12: EXTI12,
    pub exti13: EXTI13,
    pub exti14: EXTI14,
    pub exti15: EXTI15,
    pub tim1: TIM1,
    pub tim2: TIM2,
    pub tim3: TIM3,
}

pub struct BoardShared<'a> {
    pub adc1: BuddyMutex<Adc<'a, ADC1>>,
    pub usart: BuddyMutex<BufferedUart<'a>>,
}

pub struct Thermistors<'a, 'b> {
    pub bed: BuddyThermistor<'a, 'b>,
    pub board: BuddyThermistor<'a, 'b>,
    pub hotend: BuddyThermistor<'a, 'b>,
}

pub struct Steppers<'a, 'b> {
    pub x: BuddyStepperInterrupt<'a, 'b>,
    pub y: BuddyStepperInterrupt<'a, 'b>,
    pub z: BuddyStepperInterrupt<'a, 'b>,
    pub e: BuddyStepperInput<'a, 'b>,
}

pub struct Fans<'a> {
    pub fan_0: BuddyFan<'a>,
    pub fan_1: BuddyFan<'a>,
}

pub struct Heaters<'a> {
    pub bed: BuddyHeater<'a>,
    pub hotend: BuddyHeater<'a>,
}

pub struct Board<'a, 'b> {
    pub pinda_sensor: BuddyPindaSensor<'a>,
    pub filament_sensor: BuddyFilamentSensor<'a>,
    pub rotary_button: BuddyRotaryButton<'a>,
    pub rotary_encoder: BuddyRotaryEncoder<'a>,
    pub buzzer: BuddyBuzzer<'a>,
    pub thermistors: Thermistors<'a, 'b>,
    pub steppers: Steppers<'a, 'b>,
    pub fans: Fans<'a>,
    pub heaters: Heaters<'a>,
}

impl<'a, 'b> Board<'a, 'b> {
    pub async fn new(
        owned: BoardOwned,
        shared: &'a BoardShared<'b>,
    ) -> Self {
        let buzzer = Self::init_buzzer(owned.pa0, owned.tim2);
        let pinda_sensor = Self::init_pinda(owned.pa8, owned.exti8);
        let filament_sensor = Self::init_filament_sensor(owned.pb4, owned.exti4);
        let rotary_button = Self::init_rotary_button(owned.pe12, owned.exti12);
        let rotary_encoder =
            Self::init_rotary_encoder(owned.pe13, owned.exti13, owned.pe15, owned.exti15);
        let bed = Self::init_bed_thermistor(&shared.adc1, owned.pa4);
        let board = Self::init_board_thermistor(&shared.adc1, owned.pa5);
        let hotend = Self::init_hotend_thermistor(&shared.adc1, owned.pc0);
        let thermistors = Thermistors {
            board: Mutex::new(board),
            bed: Mutex::new(bed),
            hotend: Mutex::new(hotend),
        };

        let x = Self::init_x_stepper(
            owned.pd3,
            owned.pd1,
            owned.pd0,
            owned.pe2,
            owned.exti2,
            &shared.usart,
        );
        // TODO handle error
        //Self::configure_stepper(&mut x).await.unwrap();

        let y = Self::init_y_stepper(
            owned.pd14,
            owned.pd13,
            owned.pd12,
            owned.pe1,
            owned.exti1,
            &shared.usart,
        );
        // TODO handle error
        // Self::configure_stepper(&mut y).await.unwrap();

        let z = Self::init_z_stepper(
            owned.pd2,
            owned.pd4,
            owned.pd15,
            owned.pe5,
            owned.exti5,
            &shared.usart,
        );
        // TODO handle error
        //Self::configure_stepper(&mut z).await.unwrap();

        let e = Self::init_e_stepper(owned.pd10, owned.pd9, owned.pd8, owned.pa15, &shared.usart);

        let steppers = Steppers {
            x: Mutex::new(x),
            y: Mutex::new(y),
            z: Mutex::new(z),
            e: Mutex::new(e),
        };

        let fans = Self::init_fans(
            owned.pe11,
            owned.pe10,
            owned.exti10,
            owned.pe9,
            owned.pe14,
            owned.exti14,
            owned.tim1,
        );

        let heaters = Self::init_heaters(owned.pb0, owned.pb1, owned.tim3);

        Self {
            buzzer: Mutex::new(buzzer),
            pinda_sensor: Mutex::new(pinda_sensor),
            filament_sensor: Mutex::new(filament_sensor),
            rotary_button: Mutex::new(rotary_button),
            rotary_encoder: Mutex::new(rotary_encoder),
            thermistors,
            steppers,
            fans,
            heaters,
        }
    }

    pub fn init_buzzer(
        ch: PA0,
        tim: TIM2,
    ) -> Buzzer<SimplePwmChannel<'a, TIM2>> {
        let buzzer = PwmPin::new_ch1(ch, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            Some(buzzer),
            None,
            None,
            None,
            khz(21),
            Default::default(),
        );
        let mut channels = pwm.split();
        channels.ch1.enable();
        Buzzer::new(channels.ch1)
    }

    pub fn init_pinda(
        pin: PA8,
        ch: EXTI8,
    ) -> Pinda<ExtiInput<'a>> {
        let exti = ExtiInput::new(pin, ch, Pull::None);
        Pinda::new(exti)
    }

    pub fn init_filament_sensor(
        pin: PB4,
        ch: EXTI4,
    ) -> FilamentSensor<ExtiInput<'a>> {
        let exti = ExtiInput::new(pin, ch, Pull::None);
        FilamentSensor::new(exti)
    }

    pub fn init_rotary_button(
        pin: PE12,
        ch: EXTI12,
    ) -> RotaryButton<ExtiInput<'a>> {
        let exti = ExtiInput::new(pin, ch, Pull::Up);
        RotaryButton::new(exti)
    }

    pub fn init_rotary_encoder(
        pin_a: PE13,
        ch_a: EXTI13,
        pin_b: PE15,
        ch_b: EXTI15,
    ) -> RotaryEncoder<ExtiInput<'a>> {
        let extia = ExtiInput::new(pin_a, ch_a, Pull::None);
        let extib = ExtiInput::new(pin_b, ch_b, Pull::None);
        RotaryEncoder::new(extia, extib)
    }

    pub fn init_bed_thermistor(
        adc: &'a BuddyMutex<Adc<'b, ADC1>>,
        bed: PA4,
    ) -> Thermistor<'a, 'b> {
        let max_adc_sample = (1 << 12) - 1;
        Thermistor::new(
            adc,
            bed.degrade_adc(),
            4_700.0,
            max_adc_sample,
            4_092.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_board_thermistor(
        adc: &'a BuddyMutex<Adc<'b, ADC1>>,
        bed: PA5,
    ) -> Thermistor<'a, 'b> {
        let max_adc_sample = (1 << 12) - 1;
        Thermistor::new(
            adc,
            bed.degrade_adc(),
            4_700.0,
            max_adc_sample,
            4_550.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_hotend_thermistor(
        adc: &'a BuddyMutex<Adc<'b, ADC1>>,
        bed: PC0,
    ) -> Thermistor<'a, 'b> {
        let max_adc_sample = (1 << 12) - 1;
        Thermistor::new(
            adc,
            bed.degrade_adc(),
            4_700.0,
            max_adc_sample,
            4_267.0,
            100_000.0,
            25.0,
        )
    }

    pub fn init_x_stepper(
        en: PD3,
        step: PD1,
        dir: PD0,
        dia_pin: PE2,
        dia_ch: EXTI2,
        usart: &'a BuddyMutex<BufferedUart<'b>>,
    ) -> TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'b>> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new_with_interrupt(en, step, dir, dia, 1, usart).unwrap()
    }

    pub fn init_y_stepper(
        en: PD14,
        step: PD13,
        dir: PD12,
        dia_pin: PE1,
        dia_ch: EXTI1,
        usart: &'a BuddyMutex<BufferedUart<'b>>,
    ) -> TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'b>> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new_with_interrupt(en, step, dir, dia, 3, usart).unwrap()
    }

    pub fn init_z_stepper(
        en: PD2,
        step: PD4,
        dir: PD15,
        dia_pin: PE5,
        dia_ch: EXTI5,
        usart: &'a BuddyMutex<BufferedUart<'b>>,
    ) -> TMC2209AsyncUart<'a, Output<'a>, ExtiInput<'a>, ThreadModeRawMutex, BufferedUart<'b>> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = ExtiInput::new(dia_pin, dia_ch, Pull::None);
        TMC2209AsyncUart::new_with_interrupt(en, step, dir, dia, 0, usart).unwrap()
    }

    pub fn init_e_stepper(
        en: PD10,
        step: PD9,
        dir: PD8,
        dia_pin: PA15,
        usart: &'a BuddyMutex<BufferedUart<'b>>,
    ) -> TMC2209AsyncUart<'a, Output<'a>, Input<'a>, ThreadModeRawMutex, BufferedUart<'b>> {
        let en = Output::new(en, Level::High, Speed::VeryHigh);
        let step = Output::new(step, Level::High, Speed::VeryHigh);
        let dir = Output::new(dir, Level::High, Speed::VeryHigh);
        let dia = Input::new(dia_pin, Pull::None);
        TMC2209AsyncUart::new_with_input(en, step, dir, dia, 2, usart).unwrap()
    }

    pub async fn configure_stepper(
        stepper: &mut TMC2209AsyncUart<
            'a,
            Output<'a>,
            ExtiInput<'a>,
            ThreadModeRawMutex,
            BufferedUart<'b>,
        >
    ) -> Result<(), TMCError> {
        info!("Configuring Stepper Motors");
        // Configuration copied from Prusa Buddy Marlin Firmware
        // TODO: Finish the configuration porting across from their config.

        // Update the GCONF
        let mut gconf = stepper.read_gconf().await?;
        gconf.mstep_reg_select = true;
        gconf.pdn_disable = true;
        gconf.i_scale_analog = false;
        gconf.en_spreadcycle = false;

        stepper.write_register(&mut gconf).await?;

        // Update the chopconf
        let mut chopconf = stepper.read_chopconf().await?;
        chopconf.tbl = 0b01;
        chopconf.intpol = true;
        chopconf.toff = 3.into();
        chopconf.hend = 1;
        chopconf.hstrt = 5.into();
        chopconf.dedge = true;

        stepper.write_register(&mut chopconf).await?;

        let mut pwm_conf = stepper.read_pwmconf().await?;
        pwm_conf.pwm_ilm = 12;
        pwm_conf.pwm_reg = 8;
        pwm_conf.pwm_autograd = true;
        pwm_conf.pwm_autoscale = true;
        pwm_conf.pwm_freq = 0b01;
        pwm_conf.pwm_grad = 14;
        pwm_conf.pwm_ofs = 36;

        stepper.write_register(&mut pwm_conf).await?;

        Ok(())
    }

    pub fn init_fans(
        fan_0_pwm: PE11,
        fan_0_inp: PE10,
        fan_0_exti: EXTI10,
        fan_1_pwm: PE9,
        fan_1_inp: PE14,
        fan_1_exti: EXTI14,
        tim: TIM1,
    ) -> Fans<'a> {
        let fan_0_pwm_pin = PwmPin::new_ch2(fan_0_pwm, OutputType::PushPull);
        let fan_1_pwm_pin = PwmPin::new_ch1(fan_1_pwm, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            Some(fan_1_pwm_pin),
            Some(fan_0_pwm_pin),
            None,
            None,
            khz(21),
            Default::default(),
        );
        let fan_0_exti = ExtiInput::new(fan_0_inp, fan_0_exti, Pull::Down);
        let fan_1_exti = ExtiInput::new(fan_1_inp, fan_1_exti, Pull::Down);
        let mut channels = pwm.split();
        channels.ch1.enable();
        channels.ch2.enable();
        let fan_0 = Fan::new(channels.ch2, fan_0_exti);
        let fan_1 = Fan::new(channels.ch1, fan_1_exti);
        Fans {
            fan_0: Mutex::new(fan_0),
            fan_1: Mutex::new(fan_1),
        }
    }

    pub fn init_heaters(
        bed: PB0,
        hotend: PB1,
        tim: TIM3,
    ) -> Heaters<'a> {
        let bed_pin = PwmPin::new_ch3(bed, OutputType::PushPull);
        let hotend_pin = PwmPin::new_ch4(hotend, OutputType::PushPull);
        let pwm = SimplePwm::new(
            tim,
            None,
            None,
            Some(bed_pin),
            Some(hotend_pin),
            khz(21),
            Default::default(),
        );
        let channels = pwm.split();
        let bed = Heater::new(channels.ch3);
        let hotend = Heater::new(channels.ch4);
        Heaters {
            bed: Mutex::new(bed),
            hotend: Mutex::new(hotend),
        }
    }

    pub fn init_adc1(adc: ADC1) -> Adc<'a, ADC1> {
        Adc::new(adc)
    }

    pub fn config(
        p: Peripherals,
        tx_buf: &'a mut [u8; 16],
        rx_buf: &'a mut [u8; 16],
    ) -> (BoardOwned, BoardShared<'a>) {
        let owned = BoardOwned {
            pa0: p.PA0,
            pa4: p.PA4,
            pa5: p.PA5,
            pa8: p.PA8,
            pa15: p.PA15,
            pb0: p.PB0,
            pb1: p.PB1,
            pb4: p.PB4,
            pc0: p.PC0,
            pd0: p.PD0,
            pd1: p.PD1,
            pd2: p.PD2,
            pd3: p.PD3,
            pd4: p.PD4,
            pd8: p.PD8,
            pd9: p.PD9,
            pd10: p.PD10,
            pd12: p.PD12,
            pd13: p.PD13,
            pd14: p.PD14,
            pd15: p.PD15,
            pe1: p.PE1,
            pe2: p.PE2,
            pe5: p.PE5,
            pe9: p.PE9,
            pe10: p.PE10,
            pe11: p.PE11,
            pe12: p.PE12,
            pe14: p.PE14,
            pe13: p.PE13,
            pe15: p.PE15,
            exti1: p.EXTI1,
            exti2: p.EXTI2,
            exti4: p.EXTI4,
            exti5: p.EXTI5,
            exti8: p.EXTI8,
            exti10: p.EXTI10,
            exti12: p.EXTI12,
            exti13: p.EXTI13,
            exti14: p.EXTI14,
            exti15: p.EXTI15,
            tim1: p.TIM1,
            tim2: p.TIM2,
            tim3: p.TIM3,
        };
        let adc1 = Mutex::new(Board::init_adc1(p.ADC1));
        let config = Config::default();
        let usart = BufferedUart::new_half_duplex(
            p.USART2,
            p.PD5,
            Irqs,
            tx_buf,
            rx_buf,
            config,
            HalfDuplexReadback::Readback,
            HalfDuplexConfig::PushPull,
        )
        .unwrap();
        usart.set_baudrate(9_600).unwrap();
        let usart = Mutex::new(usart);
        let shared = BoardShared { adc1, usart };
        (owned, shared)
    }
}

/*
pub async fn init_thermistor_adc(adc: ADC1) -> &'a BuddyMutex<Adc<'static, ADC1>> {
    let adc = Mutex::new(Adc::new(adc));
    match BUDDY_THERMISTOR_ADC.init(adc) {
        Ok(_) => {}
        Err(_) => {
            panic!("BUDDY_THERMISTOR_ADC OnceLock Used")
        }
    }
    BUDDY_THERMISTOR_ADC.get().await
}
*/

// TODO: Still stuck on how to intialise the uart
// and then pass it through to the board. Bufs need to
// be ideally static mut given only the buf but it does
// not like that. if you create the uart then pass it through to
// the board then you need to separate the peripherals
// as creating the uart in the new() would result in self-references
// which it does not like.
// DECISION_TO_BE_MADE. Should I merge the developments together
// and stick with statics and lots of mutexes? Will I still encounter the
// same challenge of mut bufs because of the need for BufferedUart for
// the embedded hal traits to keep TMC generic across boards
/*
pub async fn init_stepper_uart(
    usart: USART2,
    tx: PD5,
    //tx_buf: &'a mut [u8],
    //rx_buf: &'a mut [u8],
) -> BuddyMutex<BufferedUart<'a>> {
    let config = Config::default();
    let usart = BufferedUart::new_half_duplex(
        usart,
        tx,
        Irqs,
        // creating a mutable reference to mutable static is discouraged.
        // I know but what can I do but some how pass out peripherals and return other structs with the items I need to build out the board?
        unsafe { &mut TX_BUF },
        unsafe { &mut RX_BUF },
        config,
        HalfDuplexReadback::Readback,
        HalfDuplexConfig::PushPull,
    )
    .unwrap();
    usart.set_baudrate(9_600).unwrap();
    Mutex::new(usart)
}
*/
