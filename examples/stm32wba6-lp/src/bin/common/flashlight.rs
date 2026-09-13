use crate::gpio_events::IonEvent;
use crate::{EventChannelSubscriber, ApplicationState, AppChannelSubscriber, TaskState};

use defmt::*;
use embassy_futures::select::{select, Either};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::peripherals::{PB8, TIM1};
use embassy_stm32::time;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::Peri;
use flashlight::*;

const FLASHLIGHT_ON_DUTY_CYCLE: u8 = 30;
const FLASHLIGHT_OFF_DUTY_CYCLE: u8 = 0;
const FLASHLIGHT_STROBE_FREQUENCY_HZ: i32 = 20;
const FLASHLIGHT_STROBE_ON_DUTY_CYCLE_PERCENT: u8 = 30;
const FLASHLIGHT_STROBE_OFF_DUTY_CYCLE_PERCENT: u8 = 0;
const FLASHLIGHT_PWM_DEFAULT_FREQUENCY_HZ: u32 = 100_000;

pub struct FlashlightHwInterface<'d> {
    pub fl_pwm: SimplePwm<'d, TIM1>,
}

impl<'d> HwInterface for FlashlightHwInterface<'d> {
    async fn set_fl_pwm_duty_cycle_percent(&mut self, duty_cycle_percent: u8) {
        self.fl_pwm.ch1().set_duty_cycle_percent(duty_cycle_percent);
    }

    async fn set_fl_pwm_frequency(&mut self, frequency: i32) {
        let freq: u32 = frequency
            .try_into()
            .unwrap_or(FLASHLIGHT_PWM_DEFAULT_FREQUENCY_HZ);
        self.fl_pwm.set_frequency(time::hz(freq));
    }

    fn enable_pwm_channel(&mut self) {
        self.fl_pwm.ch1().enable();
    }

    fn disable_pwm_channel(&mut self) {
        self.fl_pwm.ch1().disable();
    }
}

#[embassy_executor::task]
pub async fn flashlight_task(
    mut event_subscriber: EventChannelSubscriber,
    mut pin: Peri<'static, PB8>,
    mut tim1: Peri<'static, TIM1>,
    mut weapon_state_subscriber: AppChannelSubscriber,
) {

    let mut task_state = TaskState::PRESLEEP;

    loop {
        match task_state {
            TaskState::PRESLEEP => {
                task_state = TaskState::SLEEP;
                info!("flashlight_task: sleep");
            }
            TaskState::SLEEP => {
                while weapon_state_subscriber.next_message_pure().await != ApplicationState::AWAKE {}
                task_state = TaskState::PREAWAKE;
            }
            TaskState::PREAWAKE => {
                task_state = TaskState::AWAKE;
                info!("flashlight_task: awake");
            }
            TaskState::AWAKE => {
                info!("flashlight_task: initializing flashlight PWM");
                let flashlight_pwm_pin = PwmPin::new(pin.reborrow(), OutputType::PushPull);
                let mut flashlight_pwm = SimplePwm::new(
                    tim1.reborrow(),
                    Some(flashlight_pwm_pin),
                    None,
                    None,
                    None,
                    time::khz(100),
                    CountingMode::EdgeAlignedUp,
                );
                flashlight_pwm.ch1().enable();

                let flashlight_hw_interface = FlashlightHwInterface {
                    fl_pwm: flashlight_pwm,
                };

                let mut flashlight = match Flashlight::new(
                    flashlight_hw_interface,
                    FLASHLIGHT_ON_DUTY_CYCLE,
                    FLASHLIGHT_OFF_DUTY_CYCLE,
                    // FLASHLIGHT_STROBE_FREQUENCY_HZ,
                    // FLASHLIGHT_STROBE_ON_DUTY_CYCLE_PERCENT,
                    // FLASHLIGHT_STROBE_OFF_DUTY_CYCLE_PERCENT,
                ) {
                    Ok(flashlight) => flashlight,
                    Err(_e) => {
                        error!("Failed to create flashlight");
                        return;
                    }
                };

                loop {
                    match select(
                        event_subscriber.next_message_pure(),
                        weapon_state_subscriber.next_message_pure(),
                    )
                    .await
                    {
                        Either::First(ion_event) => match ion_event {
                            IonEvent::Armed => flashlight.set_mode(Mode::On).await,
                            IonEvent::Safed => flashlight.set_mode(Mode::Off).await,
                            IonEvent::ReactivatePull
                            | IonEvent::ReactivateRelease
                            | IonEvent::ReactivateBlackout => {}
                        },
                        Either::Second(ApplicationState::SLEEP) => {
                            info!("flashlight_task: WeaponState::SLEEP received");
                            flashlight.set_mode(Mode::Off).await;
                            flashlight.disable_pwm_channel();
                            break;
                        }
                        Either::Second(ApplicationState::AWAKE) => {}
                    }
                }
                drop(flashlight);
                task_state = TaskState::PRESLEEP;
            }
        }
    }
}
