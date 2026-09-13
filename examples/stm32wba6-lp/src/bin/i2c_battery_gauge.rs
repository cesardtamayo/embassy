
#![no_std]
#![no_main]

#[path = "common/helper_functions.rs"]
mod helper_functions;

use bq27441::Bq27441Async;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{GPDMA1_CH0, GPDMA1_CH2, I2C3};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, i2c, time};
use embassy_time::Timer;
use panic_probe as _;

const DEBUG_DURING_SLEEP: bool = false;

bind_interrupts!(pub struct Irqs {
    I2C3_EV => i2c::EventInterruptHandler<I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<I2C3>;
    GPDMA1_CHANNEL0 => dma::InterruptHandler<GPDMA1_CH0>;
    GPDMA1_CHANNEL2 => dma::InterruptHandler<GPDMA1_CH2>;
});

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using I2C!");

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

    let _power_rail = Output::new(p.PB15, Level::Low, Speed::Low);

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = time::khz(400); // 400 kHz I2C speed

    loop {
        let i2c = i2c::I2c::new(
            p.I2C3.reborrow(),
            p.PB2.reborrow(), // SCL
            p.PB1.reborrow(), // SDA
            p.GPDMA1_CH0.reborrow(),
            p.GPDMA1_CH2.reborrow(),
            Irqs,
            i2c_config,
        );

        match init_battery_gauge(i2c).await {
            Some(mut gauge) => {
                match gauge.voltage().await {
                    Ok(voltage_mv) => info!("battery_voltage = {} mV", voltage_mv),
                    Err(e) => error!("failed to read battery voltage: {}", defmt::Debug2Format(&e)),
                }
                drop(gauge);
            }
            None => error!("battery gauge not available"),
        }

        Timer::after_millis(5000).await;
    }
}

async fn init_battery_gauge<'d>(
    i2c_device: i2c::I2c<'d, embassy_stm32::mode::Async, i2c::mode::Master>,
) -> Option<Bq27441Async<i2c::I2c<'d, embassy_stm32::mode::Async, i2c::mode::Master>>> {
    match Bq27441Async::new(i2c_device).await {
        Ok(mut gauge) => {
            info!("BQ27441 battery gauge initialized successfully");

            // Read device info once at init (these values don't change)
            let fw_version = gauge.firmware_version().await.unwrap_or(0);
            let chem_id = gauge.chemistry_id().await.unwrap_or(0);

            info!(
                "BQ27441 FW: 0x{:04x}, Chemistry: 0x{:04x}",
                fw_version, chem_id
            );

            Some(gauge)
        }
        Err(bq27441::Error::I2c(e)) => {
            error!("I2C error: {}", e);
            None
        }
        Err(bq27441::Error::InvalidDevice) => {
            error!("Wrong device ID at 0x55");
            None
        }
        Err(bq27441::Error::InvalidParam) => {
            error!("Invalid parameter");
            None
        }
    }
}
