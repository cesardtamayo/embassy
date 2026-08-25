use crate::gpio_events::IonEvent;
use crate::EventChannelSubscriber;
use crate::ApplicationState;
use crate::AppStateChannelPublisher;
use defmt::*;
use embassy_stm32::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub async fn sleep(
    weapon_state_publisher: &AppStateChannelPublisher,
    device_pwr_en: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
) {
    weapon_state_publisher.publish_immediate(ApplicationState::SLEEP);
    let mut locked_device_pwr_en = device_pwr_en.lock().await;
    locked_device_pwr_en.set_low();
}

pub async fn awake(
    weapon_state_publisher: &AppStateChannelPublisher,
    device_pwr_en: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
) {
    let mut locked_device_pwr_en = device_pwr_en.lock().await;
    locked_device_pwr_en.set_high();
    info!("power_control: POWER HIGH ----------------- ");
    // Timer::after(Duration::from_millis(100)).await; // giving time for HV to boot
    info!("power_control: publishing AWAKE ----------------- ");
    weapon_state_publisher.publish_immediate(ApplicationState::AWAKE);
}

#[embassy_executor::task]
pub async fn power_task(
    weapon_state_publisher: AppStateChannelPublisher,
    mut io_event: EventChannelSubscriber,
    device_pwr_en: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
) {
    // awake(&weapon_state_publisher, device_pwr_en).await; // Starting with everything awake at boot.

    loop {
        match io_event.next_message().await {
            WaitResult::Lagged(_) => {}
            WaitResult::Message(IonEvent::Safed) => {
                sleep(&weapon_state_publisher, device_pwr_en).await;
            }
            WaitResult::Message(IonEvent::Armed) => {
                awake(&weapon_state_publisher, device_pwr_en).await;
            }
            WaitResult::Message(_) => {}
        }
        // Timer::after(Duration::from_secs(2)).await;
        // awake(&weapon_state_publisher, device_pwr_en).await;
        // Timer::after(Duration::from_secs(2)).await;
        // sleep(&weapon_state_publisher, device_pwr_en).await;
    }
}
