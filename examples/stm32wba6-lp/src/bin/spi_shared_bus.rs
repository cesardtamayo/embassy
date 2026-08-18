
#![no_std]
#![no_main]

#[path = "common/helper_functions.rs"]
mod helper_functions;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{GPDMA1_CH3, GPDMA1_CH4};
use embassy_stm32::interrupt::typelevel::{Interrupt, GPDMA1_CHANNEL3, GPDMA1_CHANNEL4};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::time::Hertz;
use embassy_stm32::{
    bind_interrupts,
    dma,
    spi::{Config as SpiConfig, Spi},
    time,
};
use embassy_time::{Duration, Timer};
use cat25040::{spi_device::SpiDeviceAdapter, Cat25040, HardwareInterface};
use is25lp128f::Is25lp128f;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use helper_functions::*;
use panic_probe as _;

const DEBUG_DURING_SLEEP: bool = true;

pub const READ_EEPROM_ADDR: u16 = 0;
pub const READ_FLASH_ADDR: i32 = 0;

pub struct Delay;

impl HardwareInterface for Delay {
    async fn wait_ms(&mut self, timeout_ms: u32) {
        Timer::after_millis(timeout_ms as u64).await;
    }
}

bind_interrupts!(pub struct Irqs {
    GPDMA1_CHANNEL3 => dma::InterruptHandler<GPDMA1_CH3>;
    GPDMA1_CHANNEL4 => dma::InterruptHandler<GPDMA1_CH4>;
});

pub(crate) struct FlashHardwareInterface;
impl FlashHardwareInterface {
    pub fn new() -> FlashHardwareInterface {
        FlashHardwareInterface {}
    }
}

impl is25lp128f::HardwareInterface for FlashHardwareInterface {
    async fn wait_ms(&mut self, timeout_ms: u64) {
        Timer::after(Duration::from_millis(timeout_ms)).await;
    }
}

pub(crate) struct FlashSpiAdapter<D> {
    device: D,
}

impl<D> FlashSpiAdapter<D> {
    pub fn new(device: D) -> Self {
        Self { device }
    }
}

impl<D> is25lp128f::Spi for FlashSpiAdapter<D>
where
    D: embedded_hal_async::spi::SpiDevice<u8>,
{
    async fn configure_spi(&mut self) -> Result<(), is25lp128f::Error> {
        // With SpiDevice, frequency configuration is handled at bus creation
        // or would require special handling. For now, assume bus is configured
        // at an appropriate frequency (10MHz works for most flash chips)
        Ok(())
    }

    async fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), is25lp128f::Error> {
        self.device
            .transfer_in_place(buf)
            .await
            .map_err(|_| is25lp128f::Error::SpiWriteError)
    }

    async fn read(
        &mut self,
        read_cmd_buf: &[u8],
        read_buf: &mut [u8],
    ) -> Result<(), is25lp128f::Error> {
        use embedded_hal_async::spi::Operation;

        // Use transaction to group write + read as atomic operation
        let mut operations = [Operation::Write(read_cmd_buf), Operation::Read(read_buf)];

        self.device
            .transaction(&mut operations)
            .await
            .map_err(|_| is25lp128f::Error::SpiReadError)
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), is25lp128f::Error> {
        self.device
            .write(data)
            .await
            .map_err(|_| is25lp128f::Error::SpiWriteError)
    }
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using SPI!");

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

    let _gpio_pe0 = Output::new(p.PE0, Level::Low, Speed::VeryHigh);
    let _gpio_pd14 = Output::new(p.PD14, Level::Low, Speed::VeryHigh);
    let mut flex_pd8 = Flex::new(p.PD8);
    flex_pd8.set_as_analog();
    let _gpio_ph3 = Output::new(p.PH3, Level::Low, Speed::Low);

    let _power_rail = Output::new(p.PB15, Level::Low, Speed::Low);

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = time::mhz(1);

    let mut buf = [0u8; 16];

    loop {
        // The bus only needs to be shared for the duration of this iteration, so it's
        // a plain local Mutex (not a StaticCell) built fresh from reborrowed peripherals
        // each time around - that's what lets SPI1/pins/DMA channels be reborrowed here
        // instead of requiring a single 'static init outside the loop.
        let spi = Spi::new(
            p.SPI1.reborrow(),
            p.PB4.reborrow(), // SCK
            p.PA15.reborrow(), // MOSI
            p.PB3.reborrow(), // MISO
            p.GPDMA1_CH3.reborrow(),
            p.GPDMA1_CH4.reborrow(),
            Irqs,
            spi_config,
        );
        let spi_bus = Mutex::<CriticalSectionRawMutex, _>::new(spi);

        // EEPROM
        let eeprom_cs = Output::new(p.PE1.reborrow(), Level::High, Speed::VeryHigh);
        let eeprom_spi_device = SpiDevice::new(&spi_bus, eeprom_cs);
        let eeprom_adapter = SpiDeviceAdapter::new(eeprom_spi_device);
        let mut eeprom = Cat25040::new(eeprom_adapter, Delay);

        match eeprom.read(READ_EEPROM_ADDR, &mut buf).await {
            Ok(()) => info!("eeprom buf = {}", buf),
            Err(e) => error!("eeprom read failed: {}", defmt::Debug2Format(&e)),
        }

        // FLASH
        let flash_cs = Output::new(p.PE3.reborrow(), Level::High, Speed::VeryHigh);
        let flash_spi_device = SpiDevice::new(&spi_bus, flash_cs);
        let flash_adapter = FlashSpiAdapter::new(flash_spi_device);
        let flash_hardware_interface = FlashHardwareInterface::new();
        let mut flash = Is25lp128f::new(flash_adapter, flash_hardware_interface);

        match flash.read(READ_FLASH_ADDR, &mut buf).await {
            Ok(()) => info!("flash buf = {}", buf),
            Err(e) => error!("flash read failed: {}", defmt::Debug2Format(&e)),
        }

        drop(eeprom);
        drop(flash);
        drop(spi_bus);

        // we need to restore the levels on the CS lines
        let _gpio_pe1 = Output::new(p.PE1.reborrow(), Level::High, Speed::VeryHigh);
        let _gpio_pe3 = Output::new(p.PE3.reborrow(), Level::High, Speed::VeryHigh);

        GPDMA1_CHANNEL3::unpend();
        GPDMA1_CHANNEL4::unpend();

        check_enabled_clocks();

        Timer::after_millis(5000).await;
    }
}
