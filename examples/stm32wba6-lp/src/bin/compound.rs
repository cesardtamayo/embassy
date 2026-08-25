//! Compound example: a gpio task publishes Armed/Safed events over a
//! pubsub channel, and a flashlight task subscribes to them and drives
//! the flashlight LED accordingly.
//!
//! The MCU enters STOP mode while waiting for a button press on PC13.
//! The EXTI line wakes the core from STOP — no polling required.

#![no_std]
#![no_main]

#[path = "common/flashlight.rs"]
mod flashlight;
#[path = "common/gpio_events.rs"]
mod gpio_events;
#[path = "common/power.rs"]
mod power;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pull, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use panic_probe as _;
use defmt::info;
use static_cell::StaticCell;

use crate::flashlight::flashlight_task;
use crate::gpio_events::{gpio_task, IonEvent};
use crate::power::power_task;

#[derive(Debug, PartialEq, Clone)]
pub enum ApplicationState {
    AWAKE,
    SLEEP,
}


pub enum TaskState {
    PRESLEEP,
    SLEEP,
    PREAWAKE,
    AWAKE,
}

type AppStateChannel = PubSubChannel<ThreadModeRawMutex, ApplicationState, 10, 10, 2>;
pub type AppStateChannelPublisher =
    Publisher<'static, ThreadModeRawMutex, ApplicationState, 10, 10, 2>;
pub type AppChannelSubscriber =
    Subscriber<'static, ThreadModeRawMutex, ApplicationState, 10, 10, 2>;
static APP_STATE_CHANNEL: AppStateChannel = PubSubChannel::new();

bind_interrupts!(
    pub struct Irqs {
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    }
);

type EventChannel = PubSubChannel<ThreadModeRawMutex, IonEvent, 10, 4, 4>;
pub type EventChannelPublisher = Publisher<'static, ThreadModeRawMutex, IonEvent, 10, 4, 4>;
pub type EventChannelSubscriber = Subscriber<'static, ThreadModeRawMutex, IonEvent, 10, 4, 4>;
static EVENT_CHANNEL: EventChannel = PubSubChannel::new();

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using simplePwm!");

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

        config.enable_debug_during_sleep = true;
        config.min_stop_pause = embassy_time::Duration::from_millis(10);
    }

    let p = embassy_stm32::init(config);

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

    info!("initializing power rail");
    let power_rail = Output::new(p.PB15, Level::Low, Speed::Low);

    static DEVICE_PWR_EN: StaticCell<Mutex<CriticalSectionRawMutex, Output<'static>>> =
        StaticCell::new();
    let device_pwr_en = DEVICE_PWR_EN.init(Mutex::new(power_rail));

    // STM32WBA65 has 3 types of timers:
    //  - General Purpose(TIM2/3/4/16/17)
    //  - AdvancedControl(TIM1)
    //  - LowPower(LPTIM1/2)

    let safety_s = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);

    spawner.spawn(
        gpio_task(EVENT_CHANNEL.publisher().unwrap(), safety_s)
            .expect("Failed to spawn gpio task"),
    );
    spawner.spawn(
        flashlight_task(
            EVENT_CHANNEL.subscriber().unwrap(),
            p.PB8,
            p.TIM1,
            APP_STATE_CHANNEL.subscriber().unwrap()
        )
        .expect("Failed to spawn flashlight task"),
    );

    spawner.spawn(
        power_task(
            APP_STATE_CHANNEL.publisher().unwrap(),
            EVENT_CHANNEL.subscriber().unwrap(),
            device_pwr_en,
        )
        .expect("Failed to spawn power task"),
    );

    info!("Configured gpio, flashlight, power tasks");

    loop {
        Timer::after(Duration::from_secs(2)).await;
    }
}
