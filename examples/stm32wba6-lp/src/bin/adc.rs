//! SimplePWM example with STOP mode using SimplePwm and a
//! Advance Control timer.
//!
//! The MCU enters STOP2 mode between LED toggles (5 s intervals).
//! My board draws about 160µA while sleeping, which is due to hardware
//! limitations. Actual MCU current will be measure once i test on
//! NUCLEO board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, Temperature, VrefInt, adc4};
use embassy_stm32::bind_interrupts;
use embassy_stm32::dma;
use embassy_stm32::gpio::{Flex, Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::pac::vrefbuf::vals::{Hiz, Vrs};
use embassy_stm32::peripherals::{ADC4, GPDMA1_CH1};
use embassy_stm32::time;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, OutputPolarity};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::vrefbuf::VoltageReferenceBuffer;
use embassy_time::Timer;
use panic_probe as _;

const DEBUG_DURING_SLEEP: bool = true;
const VREF_2P5V_VOLTAGE_BUFFER_SCALE: Vrs = Vrs::Vref3; // STM32WBA6 rm0515 rev 3 pg 756

#[derive(Clone)]
pub enum IonAdcChannel {
    VRefInt,
    Laser,
    LaserFlashlightTemp,
    P5P0V,
    CC1,
    CC2,
    SystemPower,
    Temperature,
    Invalid(u8),
}

impl From<u8> for IonAdcChannel {
    fn from(value: u8) -> Self {
        match value {
            0 => IonAdcChannel::VRefInt,
            1 => IonAdcChannel::Laser,
            2 => IonAdcChannel::LaserFlashlightTemp,
            3 => IonAdcChannel::P5P0V,
            4 => IonAdcChannel::CC1,
            5 => IonAdcChannel::CC2,
            6 => IonAdcChannel::SystemPower,
            7 => IonAdcChannel::Temperature,
            _ => IonAdcChannel::Invalid(0xFF),
        }
    }
}

impl From<IonAdcChannel> for u8 {
    fn from(value: IonAdcChannel) -> Self {
        match value {
            IonAdcChannel::VRefInt => 0,
            IonAdcChannel::Laser => 1,
            IonAdcChannel::LaserFlashlightTemp => 2,
            IonAdcChannel::P5P0V => 3,
            IonAdcChannel::CC1 => 4,
            IonAdcChannel::CC2 => 5,
            IonAdcChannel::SystemPower => 6,
            IonAdcChannel::Temperature => 7,
            _ => 255,
        }
    }
}

impl From<IonAdcChannel> for usize {
    fn from(ch: IonAdcChannel) -> Self {
        u8::from(ch) as usize
    }
}

bind_interrupts!(pub struct Irqs {
    GPDMA1_CHANNEL1 => dma::InterruptHandler<GPDMA1_CH1>;
});

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using ADC");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;

        // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: Some(0x0C),
        });
        // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
        config.rcc.ls = LsConfig {
            rtc: RtcClockSource::Lse,
            lsi: false,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumLow),
                peripherals_clocked: true,
            }),
        };
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),             // Fractional part (disabled)
        });

        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div1;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.apb7_pre = APBPrescaler::Div1;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;

        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
        config.rcc.mux.lptim2sel = mux::Lptim2sel::Hsi;
        config.rcc.mux.rngsel = mux::Rngsel::Hsi;
        config.rcc.sys = Sysclk::Pll1R;

        config.enable_debug_during_sleep = DEBUG_DURING_SLEEP;
        config.min_stop_pause = embassy_time::Duration::from_millis(10);
    }

    let mut p = embassy_stm32::init(config);

    info!("initializing unused GPIOs for minimum current draw ...");
    let _gpio_pd5 = Output::new(p.PD5, Level::Low, Speed::Low);
    let _gpio_pb10 = Output::new(p.PB10, Level::High, Speed::Low);
    let _gpio_pa6 = Input::new(p.PA6, Pull::Up);
    let _gpio_pe1 = Output::new(p.PE1, Level::High, Speed::VeryHigh);
    let _gpio_pe3 = Output::new(p.PE3, Level::High, Speed::VeryHigh);
    let _gpio_pe0 = Output::new(p.PE0, Level::Low, Speed::VeryHigh);
    let _gpio_pd14 = Output::new(p.PD14, Level::Low, Speed::VeryHigh);
    let mut flex_pd8 = Flex::new(p.PD8);
    flex_pd8.set_as_analog();
    let _gpio_ph3 = Output::new(p.PH3, Level::Low, Speed::Low);

    info!("initializing simplePwm and power rail");
    let mut power_rail = Output::new(p.PB15, Level::Low, Speed::Low);

    let _vrefbuf = VoltageReferenceBuffer::new(p.VREFBUF, VREF_2P5V_VOLTAGE_BUFFER_SCALE, Hiz::Connected);

    let mut measurements = [0u16; 8];
    let mut cc_1_sample = 0;
    let mut cc_2_sample = 0;

    loop {
        info!("initializing ADC4");
        let mut adc = Adc::new_adc4(p.ADC4.reborrow());
        let mut adc_dma = p.GPDMA1_CH1.reborrow();

        let mut vrefint = adc.enable_vrefint_adc4();
        let mut temperature = adc.enable_temperature_adc4();
        let mut laser_sns = p.PA7.reborrow();
        let mut laser_fl_temp_sns = p.PA5.reborrow();
        let mut cc_1 = p.PA1.reborrow();
        let mut cc_2 = p.PA0.reborrow();
        let mut sys_pwr_sns = p.PB9.reborrow();
        let mut p5p0v_sns = p.PA2.reborrow();

        adc.set_resolution_adc4(adc4::Resolution::Bits12);
        adc.set_averaging_adc4(adc4::Averaging::Samples32);
        let sample_time = adc4::SampleTime::Cycles795;

        let sequence = [
            (vrefint.reborrow_adc(), sample_time),
            (laser_sns.reborrow_adc(), sample_time),
            (laser_fl_temp_sns.reborrow_adc(), sample_time),
            (p5p0v_sns.reborrow_adc(), sample_time),
            (cc_1.reborrow_adc(), sample_time),
            (cc_2.reborrow_adc(), sample_time),
            (sys_pwr_sns.reborrow_adc(), sample_time),
            (temperature.reborrow_adc(), sample_time),
        ]
        .into_iter();
        adc.read(adc_dma.reborrow(), Irqs, sequence, None, &mut measurements)
            .await;

        cc_1_sample = measurements[usize::from(IonAdcChannel::CC1)] as i16;
        cc_2_sample = measurements[usize::from(IonAdcChannel::CC2)] as i16;

        info!("ADC sampled: cc1 = {}; cc2 = {}", cc_1_sample, cc_2_sample);
        info!("mocking some delay to see current difference...");
        Timer::after_millis(500).await;

        drop(adc);
        // drop(adc_dma);
        info!("sleeping for 5s ...");
        Timer::after_millis(5000).await;
    }
}
