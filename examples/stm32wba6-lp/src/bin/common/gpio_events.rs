use crate::EventChannelPublisher;
// ucarog se debounced_button::{DebouncedButton, DebouncedEdge, DebouncedLevel};
use embassy_stm32::exti::ExtiInput;
// use embassy_futures::select::{Either, select};
// use embassy_stm32::gpio;
// use embassy_stm32::gpio::Input;
use embassy_stm32::mode::Async;
// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use defmt::info;
// use safety_slider::{Edge, SafetySlider, SliderEvent};
// use switch_blackout::{Blackout, Event as BlackoutEvent, Signal};

#[derive(Debug, defmt::Format, PartialEq, Clone, Copy)]
pub enum IonEvent {
    Safed,
    Armed,
    ReactivatePull,
    ReactivateRelease,
    ReactivateBlackout,
}

// #[derive(Debug, defmt::Format, PartialEq, Clone, Copy)]
// pub enum Level {
//     High,
//     Low,
//     Initial,
// }

// impl From<gpio::Level> for Level {
//     fn from(value: gpio::Level) -> Self {
//         match value {
//             gpio::Level::High => Level::High,
//             gpio::Level::Low => Level::Low,
//         }
//     }
// }

// impl From<Level> for DebouncedLevel {
//     fn from(value: Level) -> Self {
//         match value {
//             Level::High => DebouncedLevel::High,
//             Level::Low => DebouncedLevel::Low,
//             Level::Initial => DebouncedLevel::Initial,
//         }
//     }
// }

// impl From<Level> for DebouncedEdge {
//     fn from(value: Level) -> Self {
//         match value {
//             Level::High => DebouncedEdge::Rising,
//             Level::Low => DebouncedEdge::Falling,
//             Level::Initial => DebouncedEdge::Initial,
//         }
//     }
// }

// pub struct Buttons {
//     pub safety_s: DebouncedButton,
//     pub reactivate: DebouncedButton,
// }

// impl Buttons {
//     pub fn new() -> Self {
//         Self {
//             safety_s: DebouncedButton::new(DEBOUNCE_LIMIT_MS),
//             reactivate: DebouncedButton::new(DEBOUNCE_LIMIT_MS),
//         }
//     }
// }

// const DEBOUNCE_LIMIT_MS: i32 = 8;
// const SAFETY_SLIDER_THRESHOLD_MS: u32 = 100;
// const ARMED_REACTIVATE_BLACKOUT_MS: u32 = 100;

// fn debounced_to_edge(edge: &DebouncedEdge) -> Edge {
//     match edge {
//         DebouncedEdge::Rising => Edge::Rising,
//         DebouncedEdge::Falling => Edge::Falling,
//         _ => Edge::None,
//     }
// }

// enum TaskState {
//     Sleep,
//     Awake,
// }

#[embassy_executor::task]
pub async fn gpio_task(
    event_publisher: EventChannelPublisher,
    mut safety_s: ExtiInput<'static, Async>,
) {

    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("gpio_task: publishing Armed");
        event_publisher.publish_immediate(IonEvent::Armed);

        Timer::after(Duration::from_secs(3)).await;
        info!("gpio_task: publishing Safed");
        event_publisher.publish_immediate(IonEvent::Safed);

    }
}
