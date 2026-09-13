
#![no_std]
#![no_main]

#[path = "common/helper_functions.rs"]
mod helper_functions;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pull, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::dma;
use embassy_time::{with_timeout, Duration, Timer};
use embassy_stm32::peripherals::{GPDMA1_CH5, GPDMA1_CH6, USART3};
use embassy_stm32::usart;
use embassy_stm32::interrupt::typelevel::{Interrupt, GPDMA1_CHANNEL5, GPDMA1_CHANNEL6, USART3 as UsartIrq};
use embassy_stm32::bind_interrupts;
use embassy_stm32::pac;
use embedded_io_async::Write as _;
use panic_probe as _;
use static_cell::StaticCell;
use crate::helper_functions::*;

// TEST: temporarily false to check whether keeping the debug clock alive during
// STOP2 (so RTT logging survives sleep) is itself causing the "woke from
// STOP0/1" bounce we've been chasing - no pending IRQ was found on any
// checked peripheral, so this is the next most likely deterministic cause.
const DEBUG_DURING_SLEEP: bool = true;

bind_interrupts!(pub struct Irqs {
    USART3 => usart::InterruptHandler<USART3>;
    GPDMA1_CHANNEL5 => dma::InterruptHandler<GPDMA1_CH5>;
    GPDMA1_CHANNEL6 => dma::InterruptHandler<GPDMA1_CH6>;
});

static UART_RX_RING_BUF: StaticCell<[u8; 8192]> = StaticCell::new();

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using UART!");

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

    use embassy_stm32::usart::Config as UsartConfig;
    let mut hv_uart_config = UsartConfig::default();
    hv_uart_config.rx_pull = Pull::Up;

    // StaticCell::init() can only be called once, so grab the 'static buffer
    // up front and reborrow it each loop iteration instead of re-initializing.
    let uart_rx_buf = UART_RX_RING_BUF.init([0u8; 8192]);
    let test_serial_data = [0xa, 0xb, 0xc]; //build_get_version_request();

    loop {

        let (mut hv_uart_tx, hv_uart_rx) = embassy_stm32::usart::Uart::new(
            p.USART3.reborrow(),
            p.PC5.reborrow(),        // RX
            p.PC4.reborrow(),        // TX
            p.GPDMA1_CH5.reborrow(), // TX DMA
            p.GPDMA1_CH6.reborrow(), // RX DMA
            Irqs,
            hv_uart_config,
        )
        .unwrap()
        .split();

        let mut hv_uart_rx = hv_uart_rx.into_ring_buffered(&mut uart_rx_buf[..]);

        match hv_uart_tx.write_all(&test_serial_data).await {
            Ok(_) => info!("wrote get_version request: {:02x}", test_serial_data),
            Err(e) => {
                error!("Failed to write to UART: {:?}", defmt::Debug2Format(&e));
                break;
            }
        }

        let mut buf = [0u8; 64];
        match with_timeout(Duration::from_millis(300), hv_uart_rx.read(&mut buf)).await {
            Ok(Ok(0)) => {
                warn!("read returned 0 bytes");
            }
            Ok(Ok(n)) => {
                info!("got {} bytes: {:02x}", n, &buf[..n]);
            }
            Ok(Err(e)) => {
                error!(
                    "error reading from UART: {:?}",
                    defmt::Debug2Format(&e)
                );
            }
            Err(_) => {
                warn!("timed out waiting for HV response");
            }
        }
        drop(hv_uart_tx);
        drop(hv_uart_rx);
        for ch in [5, 6] {
            pac::GPDMA1.ch(ch).fcr().write(|w| {
                w.set_dtef(true);
                w.set_htf(true);
                w.set_suspf(true);
                w.set_tcf(true);
                w.set_tof(true);
                w.set_ulef(true);
                w.set_usef(true);
            });
        }


        GPDMA1_CHANNEL5::unpend();
        GPDMA1_CHANNEL6::unpend();

        crate::pac::RCC.ahb1enr().modify(|w| w.set_gpdma1en(false));

        // // The isolation test (UART/DMA fully disabled) went straight to real STOP2
        // // with no spurious wake, so the DMA fcr-clear above isn't the deciding
        // // factor - try clearing USART3's own latched flags too, mirroring
        // // clear_interrupt_flags(): write the ISR value back into ICR to clear
        // // whatever's currently set (e.g. IDLE, re-armed by check_idle_and_errors
        // // on every successful read, right before stop_uart() disables IDLEIE).
        let isr = pac::USART3.isr().read();
        pac::USART3.icr().write(|w| *w = pac::usart::regs::Icr(isr.0));
        UsartIrq::unpend();

        // // DIAGNOSTIC: neither fix above changed anything, so ask the NVIC directly
        // // what's actually pending right before we try to sleep, instead of guessing.
        // use embassy_stm32::interrupt as irq;
        // use cortex_m::peripheral::NVIC;
        // for (name, i) in [
        //     ("USART3", irq::USART3),
        //     ("GPDMA1_CHANNEL0", irq::GPDMA1_CHANNEL0),
        //     ("GPDMA1_CHANNEL1", irq::GPDMA1_CHANNEL1),
        //     ("GPDMA1_CHANNEL2", irq::GPDMA1_CHANNEL2),
        //     ("GPDMA1_CHANNEL3", irq::GPDMA1_CHANNEL3),
        //     ("GPDMA1_CHANNEL4", irq::GPDMA1_CHANNEL4),
        //     ("GPDMA1_CHANNEL5", irq::GPDMA1_CHANNEL5),
        //     ("GPDMA1_CHANNEL6", irq::GPDMA1_CHANNEL6),
        //     ("GPDMA1_CHANNEL7", irq::GPDMA1_CHANNEL7),
        // ] {
        //     if NVIC::is_pending(i) {
        //         warn!("PENDING IRQ: {}", name);
        //     }
        // }

        check_enabled_clocks();

        Timer::after_millis(5000).await;

        crate::pac::RCC.ahb1enr().modify(|w| w.set_gpdma1en(true));
    }
}
