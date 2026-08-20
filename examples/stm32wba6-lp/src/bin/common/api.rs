// use crate::adc::{IonAdcChannel, SharedAdcValues};
// use crate::ble::BLE_DTM_RESULT;
// use crate::device_settings::SharedDeviceSettings;
// use crate::eeprom::*;
// use crate::gpio_events::{self, Buttons, IonEvent, Level};
// use crate::hv_api::HvApiHandler;
// use crate::led::*;
// use crate::lifetime_stats::SharedLifetimeStats;
// use crate::logger::{LogEraseReq, LogFlushReq, LogReadReq, LogReadRsp, LoggerCtrl, LOG_CTRL};
// use crate::product_info::SharedProductInfo;
// use crate::sensors::SharedSensorData;
// use crate::{
//     CliEventChannelPublisher, EnqueuedEvent, EventChannelPublisher, SimplePwm,
//     WeaponStateChannelPublisher,
// };
use core::mem;
// use core::ptr;
// use cortex_m::peripheral::SCB;
// use crc16;
use defmt::*;
use embassy_futures::select::{select, Either};
// use embassy_stm32::flash::{Blocking, Error, Flash};
// use embassy_stm32::gpio::Output;
// use embassy_stm32::lptim::pwm::Pwm;
// use embassy_stm32::pac;
// use embassy_stm32::pac::spi::vals::Comm;
// use embassy_stm32::pac::DESIG;
// use embassy_stm32::peripherals::{LPTIM2, TIM16, TIM2};
// use embassy_stm32::time;
// use embassy_stm32::uid;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{Read, Write};
// use ion_types::{
//     self, magazine_type, BuzzerProfile, Command, LSR_calibration_table,
//     LSR_calibration_table_slice, LSR_state, LogsEraseRsp, LogsFlushRsp, LogsGetRsp, Response,
// };
// use led::*;
// use logger::Error as LoggerError;
// use magazine_slider::*;
// use memory::{BANK1_BUILD, BANK2_BUILD, BOOTLOADER_BUILD};
use static_cell::StaticCell;
use crate::UsbIoError;

const RSP_BUF_SIZE: usize = 8200;
const LOGS_READ_BUF_SIZE: usize = 8192;

// const LASER_CALIBRATION_SIZE: usize = mem::size_of::<LSR_calibration_table_slice>();

// #[derive(Debug, PartialEq, Clone)]
// pub enum IonCliEvent {
//     LaserSet {
//         state: LSR_state,
//     },
//     LaserGetCalibrationTable {},
//     LaserSetCalibrationTable {
//         calibration_table: LSR_calibration_table,
//     },
//     FlashlightStrobeSet {
//         strobe_frequency: i32,
//         strobe_on_duty_cycle_percent: u8,
//         strobe_off_duty_cycle_percent: u8,
//     },
//     FlashlightPwmSet {
//         frequency: i32,
//         duty_cycle_percent: u8,
//     },
//     BuzzerProfileSet {
//         profile: BuzzerProfile,
//     },
//     BleDtmStartTx {
//         channel: i8,
//         length: i8,
//         payload: ion_types::BleDtmPayloadType,
//     },
//     BleDtmStartRx {
//         channel: i8,
//         length: i8,
//     },
//     BleDtmStop {},
// }

// static COMMAND: StaticCell<Command> = StaticCell::new();
static CMD_BUF: StaticCell<[u8; 128]> = StaticCell::new();
// static RSP_BUF: StaticCell<[u8; RSP_BUF_SIZE]> = StaticCell::new();

// static LOGS_READ_REPLY: StaticCell<Signal<CriticalSectionRawMutex, LogReadRsp>> = StaticCell::new();
// static LOGS_ERASE_REPLY: StaticCell<Signal<CriticalSectionRawMutex, Result<(), LoggerError>>> =
//     StaticCell::new();
// static LOGS_FLUSH_REPLY: StaticCell<Signal<CriticalSectionRawMutex, Result<(), LoggerError>>> =
//     StaticCell::new();
// static LOGS_READ_BUF: StaticCell<[u8; LOGS_READ_BUF_SIZE]> = StaticCell::new();

pub struct ApiHandler<S>
where
    S: Read + Write + Unpin,
{
    pub serial: S,
    // on_chip_flash: &'a mut Flash<'static, Blocking>, // TODO: Remove ownership
    // hv_api_mutex: &'static Mutex<CriticalSectionRawMutex, HvApiHandler>,
    cmd_buf: &'static mut [u8; 128],
    rec_len: usize,
    rec_stale: Instant,
    // rsp_buf: &'static mut [u8; RSP_BUF_SIZE],
    // rsp_len: usize,
    // cli_event_publisher: &'a CliEventChannelPublisher,
    // event_publisher: &'a EventChannelPublisher,
    // shared_adc_values: &'static SharedAdcValues,
    // shared_gpio_states: &'static Mutex<CriticalSectionRawMutex, Buttons>,
    // command: &'static mut Command,
    // audio_pwm_mutex: &'static Mutex<CriticalSectionRawMutex, Pwm<'static, LPTIM2>>,
    // device_pwr_en_mutex: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
    // vbus_en_mutex: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
    // battery_led_hw: BatteryLedHardware,
    // ble_led_hw: BleLedHardware,
    // shared_eeprom: &'static SharedEeprom,
    // shared_flash: &'static crate::off_chip_flash::SharedFlash,
    // shared_sensor_data: &'static SharedSensorData,
    // product_info: SharedProductInfo,
    // lifetime_stats: SharedLifetimeStats,
    // laser_fixed_mutex: &'static Mutex<CriticalSectionRawMutex, SimplePwm<'static, TIM2>>,
    // laser_variable_mutex: &'static Mutex<CriticalSectionRawMutex, SimplePwm<'static, TIM16>>,
    // logs_read_reply: &'static Signal<CriticalSectionRawMutex, LogReadRsp>,
    // logs_erase_reply: &'static Signal<CriticalSectionRawMutex, Result<(), LoggerError>>,
    // logs_flush_reply: &'static Signal<CriticalSectionRawMutex, Result<(), LoggerError>>,
    // logs_read_buf: Option<&'static mut [u8; LOGS_READ_BUF_SIZE]>,
    // magazine_mutex: &'static Mutex<CriticalSectionRawMutex, Magazine>,
    // device_settings: SharedDeviceSettings,
    // weapon_state_publisher: &'a WeaponStateChannelPublisher,
}

impl<S> ApiHandler<S>
where
    S: Read + Write + Unpin + embedded_io_async::ErrorType<Error = UsbIoError>,
    <S as embedded_io_async::ErrorType>::Error: defmt::Format,
{
    pub fn new(
        serial: S,
        // on_chip_flash: &'a mut Flash<'static, Blocking>,
        // hv_api_mutex: &'static Mutex<CriticalSectionRawMutex, HvApiHandler>,
        // cli_event_publisher: &'a CliEventChannelPublisher,
        // event_publisher: &'a EventChannelPublisher,
        // shared_adc_values: &'static SharedAdcValues,
        // shared_gpio_states: &'static Mutex<CriticalSectionRawMutex, Buttons>,
        // audio_pwm_mutex: &'static Mutex<CriticalSectionRawMutex, Pwm<'static, LPTIM2>>,
        // device_pwr_en_mutex: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
        // vbus_en_mutex: &'static Mutex<CriticalSectionRawMutex, Output<'static>>,
        // battery_led_hw: BatteryLedHardware,
        // ble_led_hw: BleLedHardware,
        // shared_eeprom: &'static SharedEeprom,
        // shared_flash: &'static crate::off_chip_flash::SharedFlash,
        // shared_sensor_data: &'static SharedSensorData,
        // product_info: SharedProductInfo,
        // lifetime_stats: SharedLifetimeStats,
        // laser_fixed_mutex: &'static Mutex<CriticalSectionRawMutex, SimplePwm<'static, TIM2>>,
        // laser_variable_mutex: &'static Mutex<CriticalSectionRawMutex, SimplePwm<'static, TIM16>>,
        // magazine_mutex: &'static Mutex<CriticalSectionRawMutex, Magazine>,
        // device_settings: SharedDeviceSettings,
        // weapon_state_publisher: &'a WeaponStateChannelPublisher,
    ) -> Self {
        ApiHandler {
            serial,
            // on_chip_flash,
            // hv_api_mutex,
            cmd_buf: CMD_BUF.init([0u8; 128]),
            rec_len: 0,
            rec_stale: Instant::now(),
            // rsp_buf: RSP_BUF.init([0u8; RSP_BUF_SIZE]),
            // rsp_len: 0,
            // cli_event_publisher,
            // event_publisher,
            // shared_adc_values,
            // shared_gpio_states,
            // command: COMMAND.init(Command::NoCommand(ion_types::NoCommand {})),
            // audio_pwm_mutex,
            // device_pwr_en_mutex,
            // vbus_en_mutex,
            // battery_led_hw,
            // ble_led_hw,
            // shared_eeprom,
            // shared_flash,
            // shared_sensor_data,
            // product_info,
            // lifetime_stats,
            // laser_fixed_mutex,
            // laser_variable_mutex,
            // logs_read_reply: LOGS_READ_REPLY.init(Signal::new()),
            // logs_erase_reply: LOGS_ERASE_REPLY.init(Signal::new()),
            // logs_flush_reply: LOGS_FLUSH_REPLY.init(Signal::new()),
            // logs_read_buf: Some(LOGS_READ_BUF.init([0u8; LOGS_READ_BUF_SIZE])),
            // magazine_mutex,
            // device_settings,
            // weapon_state_publisher,
        }
    }

    async fn read_data(&mut self) -> Result<usize, UsbIoError> {
        let read_slice = self.cmd_buf[self.rec_len..].as_mut();
        match Read::read(&mut self.serial, read_slice).await {
            Ok(r) => {
                self.rec_stale = Instant::now() + Duration::from_millis(800);
                debug!("Added {} bytes for {} total", r, self.rec_len + 1);
                Ok(r)
            }
            Err(e) => {
                error!("Error reading from serial port `{}`", e);
                self.flush_cmd_buf();
                Timer::after(Duration::from_millis(500)).await;
                Err(e)
            }
        }
    }

    pub async fn receive(&mut self) -> Result<bool, UsbIoError> {
        let stale_timeout = self.rec_stale;

        let rec_len = if let Either::First(future) =
            select(self.read_data(), Timer::at(stale_timeout)).await
        {
            match future {
                Ok(r) => r,
                Err(e) => return Err(e),
            }
        } else if self.rec_len > 0 {
            error!("API command is stale");
            self.flush_cmd_buf();
            return Ok(false);
        } else {
            self.flush_cmd_buf();
            0
        };

        self.rec_len += rec_len;
        Ok(true)
    }

    pub fn get_rec_len(&mut self) -> usize {
        self.rec_len
    }

    // fn process_command(&mut self) -> Result<bool, UsbIoError> {
    //     let command_tag = self.cmd_buf[0];
    //     let expected_len = self.get_expected_length(command_tag);

    //     if self.rec_len < expected_len {
    //         debug!("Waiting for more bytes");
    //         return Ok(false); // Not ready yet
    //     }

    //     match Command::try_from(&self.cmd_buf[..self.rec_len]) {
    //         Ok(cmd) => {
    //             *self.command = cmd;
    //             Ok(true) // Command is ready for execution
    //         }
    //         Err(_) => {
    //             self.flush_cmd_buf();
    //             Ok(false)
    //         }
    //     }
    // }

    // fn get_expected_length(&mut self, command_tag: u8) -> usize {
    //     match Command::size_from_tag(command_tag) {
    //         Some(expected_length) => expected_length,
    //         None => 0,
    //     }
    // }

    // pub async fn send(&mut self) {
    //     if self.rsp_len > 0 {
    //         let mut offset = 0;
    //         let _rsp = &self.rsp_buf[..self.rsp_len];
    //         while offset < self.rsp_len {
    //             offset += match Write::write(&mut self.serial, &self.rsp_buf[offset..self.rsp_len])
    //                 .await
    //             {
    //                 Ok(n) => n,
    //                 Err(e) => {
    //                     error!("Failed to write to serial: {:?}", defmt::Debug2Format(&e));
    //                     return;
    //                 }
    //             };
    //         }
    //         self.flush_cmd_buf();
    //         self.flush_send_buf();
    //     }
    // }

    // pub async fn execute(&mut self) {
    //     match self.command {
    //         Command::GetBankVersion(_) => {
    //             self.get_bank_version().await;
    //         }
    //         Command::GetSense(_) => {
    //             self.get_sense().await;
    //         }
    //         Command::Reset(_) => {
    //             self.soft_reset().await;
    //         }
    //         Command::OnChipFlashWrite(_) => {
    //             self.on_chip_flash_write();
    //         }
    //         Command::OnChipFlashRead(_) => {
    //             self.on_chip_flash_read();
    //         }
    //         Command::NoCommand(_) => {}
    //         Command::GetVersion(_) => {
    //             self.get_version(None).await;
    //         }
    //         Command::StatusSet(_) => {
    //             self.set_status().await;
    //         }
    //         Command::HvStatusSet(_) => {
    //             self.set_hv_bank_status().await;
    //         }
    //         Command::HvFlashWrite(_) => {
    //             self.hv_flash_write().await;
    //         }
    //         Command::BuzzerSet(_) => {
    //             self.buzzer_set().await;
    //         }
    //         Command::HvReset(_) => {
    //             self.hv_reset().await;
    //         }
    //         Command::DevicePower(_) => {
    //             self.device_power_set().await;
    //         }
    //         Command::CartridgeForceFire(_) => {
    //             self.force_fire_cartridge().await;
    //         }
    //         Command::NmiStart(_) => {
    //             self.nmi_start().await;
    //         }
    //         Command::FlashlightSet(_) => {
    //             self.flashlight_set().await;
    //         }
    //         Command::LedSet(_) => {
    //             self.led_set().await;
    //         }
    //         Command::LaserSet(_) => {
    //             self.laser_set().await;
    //         }
    //         Command::LaserGetCalibrationTable(_) => {
    //             self.laser_get_calibration_table().await;
    //         }
    //         Command::LaserSetCalibrationTable(_) => {
    //             self.laser_set_calibration_table().await;
    //         }
    //         Command::EepromWrite(_) => {
    //             self.eeprom_write().await;
    //         }
    //         Command::EepromRead(_) => {
    //             self.eeprom_read().await;
    //         }
    //         Command::FlashlightStrobeSet(_) => {
    //             self.flashlight_strobe_set().await;
    //         }
    //         Command::BuzzerProfileSet(_) => {
    //             self.buzzer_profile_set().await;
    //         }
    //         Command::OffChipFlashErase(_) => {
    //             self.off_chip_flash_erase().await;
    //         }
    //         Command::OffChipFlashWrite(_) => {
    //             self.off_chip_flash_write().await;
    //         }
    //         Command::OffChipFlashRead(_) => {
    //             self.off_chip_flash_read().await;
    //         }
    //         Command::OffChipFlashReadProductId(_) => {
    //             self.off_chip_flash_read_product_id().await;
    //         }
    //         Command::TimeSet(_) => {
    //             self.time_set().await;
    //         }
    //         Command::TimeGet(_) => {
    //             self.time_get().await;
    //         }
    //         Command::GetGaugeInfo(_) => {
    //             self.get_gauge_info();
    //         }
    //         Command::UniqueIdsRead(_) => {
    //             self.read_unique_ids().await;
    //         }
    //         Command::ProductInfoGet(_) => {
    //             self.product_info_get().await;
    //         }
    //         Command::ProductInfoSet(_) => {
    //             self.product_info_set().await;
    //         }
    //         Command::LifetimeStatsGet(_) => {
    //             self.lifetime_stats_get().await;
    //         }
    //         Command::LogsGet(_) => {
    //             self.logs_get().await;
    //         }
    //         Command::LogsErase(_) => {
    //             self.logs_erase().await;
    //         }
    //         Command::LogsFlush(_) => {
    //             self.logs_flush().await;
    //         }
    //         Command::IonEvent(_) => {
    //             self.publish_ion_event().await;
    //         }
    //         Command::FlashRdpOptionRead(_) => {
    //             self.read_flash_rdp_option();
    //         }
    //         Command::BleDtmStartTx(_) => {
    //             self.ble_dtm_start_tx().await;
    //         }
    //         Command::BleDtmStartRx(_) => {
    //             self.ble_dtm_start_rx().await;
    //         }
    //         Command::BleDtmStop(_) => {
    //             self.ble_dtm_stop().await;
    //         }
    //         Command::DeviceSettingsGet(_) => {
    //             self.device_settings_get().await;
    //         }
    //         Command::DeviceSettingsSet(_) => {
    //             self.device_settings_set().await;
    //         }
    //         Command::HvArm(_) => {
    //             self.hv_arm().await;
    //         }
    //     }
    // }

    // async fn publish_ion_event(&mut self) {
    //     let opts = match &*self.command {
    //         Command::IonEvent(opts) => opts,
    //         _ => {
    //             error!("publish_ionEvent called but current command is not IonEvent");
    //             return;
    //         }
    //     };

    //     self.event_publisher.publish_immediate(
    //         EnqueuedEvent::from_cli(opts.Event).expect("Failed to convert event from CLI."),
    //     );

    //     let rsp = Response::IonEvent(ion_types::IonEventRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn set_status(&mut self) {
    //     let opts = match &*self.command {
    //         Command::StatusSet(opts) => opts,
    //         _ => {
    //             error!("set_status called but current command is not StatusSet");
    //             return;
    //         }
    //     };
    //     let status_address = match opts.bank {
    //         0 => memory::BOOTLOADER_STATUS,
    //         1 => memory::BANK1_STATUS,
    //         2 => memory::BANK2_STATUS,
    //         _ => {
    //             error!("Invalid Bank: Bank {}", opts.bank);
    //             return;
    //         }
    //     };

    //     let status = unsafe { ptr::read_volatile(status_address as *const u64) };
    //     debug!("Old Status: {}", status);
    //     let update = u64::from_le_bytes(opts.status.clone().into());
    //     debug!("Update Status: {}", update);
    //     let new_status = status & update;
    //     debug!("New Status: {}", new_status);
    //     let new_status: [u8; 8] = new_status.to_le_bytes();

    //     let offset = status_address as u32 - memory::FLASH_BASE as u32;
    //     let mut page_buf = [0u8; 8192];
    //     match flash_utils::edit_page::<8192, _>(
    //         self.on_chip_flash,
    //         offset,
    //         &new_status,
    //         &mut page_buf,
    //     ) {
    //         Ok(_) => {
    //             self.get_version(Some(opts.bank)).await;
    //         }
    //         Err(e) => {
    //             error!("Failed to edit status: {:?}", defmt::Debug2Format(&e));
    //         }
    //     }
    // }

    // async fn set_hv_bank_status(&mut self) {
    //     let opts = match &*self.command {
    //         Command::HvStatusSet(opts) => opts,
    //         _ => {
    //             error!("set_hv_bank_status called but current command is not HvStatusSet");
    //             return;
    //         }
    //     };
    //     {
    //         let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //         let _ = locked_hv_api
    //             .set_bank_status(
    //                 opts.bank,
    //                 opts.status.verified == ion_types::APHD_verified::verified,
    //                 opts.status.rejected == ion_types::APHD_rejected::rejected,
    //                 opts.status.tested == ion_types::APHD_tested::tested,
    //             )
    //             .await
    //             .unwrap_or_else(|_| ());
    //     }
    //     let rsp = Response::HvStatusSet(ion_types::HvStatusSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // fn get_this_bank() -> i8 {
    //     let addr = Self::get_this_bank as isize;
    //     if addr >= memory::BANK2_BASE && addr < memory::BANK2_END {
    //         2
    //     } else if addr >= memory::BANK1_BASE && addr < memory::BANK1_END {
    //         1
    //     } else {
    //         0
    //     }
    // }

    // async fn get_bank_version(&mut self) {
    //     let bank = match &*self.command {
    //         Command::GetBankVersion(opts) => opts.bank,
    //         _ => {
    //             error!("get_bank_version called but current command is not GetBankVersion");
    //             return;
    //         }
    //     };
    //     self.get_version(Some(bank)).await;
    // }

    // async fn get_sense(&mut self) {
    //     let mut data = [0i16; 8];

    //     for i in 0..data.len() {
    //         data[i] = self
    //             .shared_adc_values
    //             .get_adc_value_raw(IonAdcChannel::from(i as u8));
    //     }

    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     let hv_sense = locked_hv_api.sense_all().await.unwrap_or_else(|_| {
    //         thunderbird_types::API_rsp_sense_all_ion {
    //             cart_1_counts: -1,
    //             cart_1_state: thunderbird_types::CART_state::max,
    //             cart_2_counts: -1,
    //             cart_2_state: thunderbird_types::CART_state::max,
    //             cart_3_counts: -1,
    //             cart_3_state: thunderbird_types::CART_state::max,
    //             cart_4_counts: -1,
    //             cart_4_state: thunderbird_types::CART_state::max,
    //             p_cap_sns_counts: -1,
    //             hv_pos_volts: -1,
    //             p_msc_sns_counts: -1,
    //             p_msc_volts: -1,
    //             n_msc_sns_counts: -1,
    //             n_msc_volts: -1,
    //             pbat_sns_counts: -1,
    //             pbat_sns_mv: -1,
    //             pbias_sns_counts: -1,
    //             pbias_sns_mv: -1,
    //             p3p3v_sns_counts: -1,
    //             p3p3v_sns_mv: -1,
    //         }
    //     });
    //     let button_states = self.shared_gpio_states.lock().await;
    //     let vbus_en = self.vbus_en_mutex.lock().await.is_set_high();
    //     let magazine_type = self.get_magazine_type().await;

    //     let rsp: ion_types::GetSenseRsp = ion_types::GetSenseRsp {
    //         vrefint: data[0],
    //         vrefint_mv: self
    //             .shared_adc_values
    //             .get_adc_value_converted(IonAdcChannel::VRefInt),
    //         laser_sns: data[1],
    //         laser_fl_temp_sns: data[2],
    //         laser_fl_temp_c: self
    //             .shared_adc_values
    //             .get_adc_value_converted(IonAdcChannel::LaserFlashlightTemp),
    //         laser_fixed_pwm_duty_cycle: self
    //             .laser_fixed_mutex
    //             .lock()
    //             .await
    //             .ch1()
    //             .current_duty_cycle() as i32,
    //         laser_variable_pwm_duty_cycle: self
    //             .laser_variable_mutex
    //             .lock()
    //             .await
    //             .ch1()
    //             .current_duty_cycle() as i32,
    //         p5p0v_sns: data[3],
    //         p5p0v_mv: self
    //             .shared_adc_values
    //             .get_adc_value_converted(IonAdcChannel::P5P0V),
    //         cc_1: data[4],
    //         cc_2: data[5],
    //         sys_pwr_sns: data[6],
    //         sys_pwr_mv: self
    //             .shared_adc_values
    //             .get_adc_value_converted(IonAdcChannel::SystemPower),
    //         vbus_en: vbus_en,
    //         mcu_temperature: data[7],
    //         mcu_temperature_c: self
    //             .shared_adc_values
    //             .get_adc_value_converted(IonAdcChannel::Temperature),
    //         // GPIO states
    //         trigger: button_states.get_state(gpio_events::Inputs::Trigger) == Level::High,
    //         reactivate: button_states.get_state(gpio_events::Inputs::Reactivate) == Level::High,
    //         safety_s: button_states.get_state(gpio_events::Inputs::SafetyS) == Level::High,
    //         mag_2_n: button_states.get_state(gpio_events::Inputs::Mag2N) == Level::High,
    //         mag_2_s: button_states.get_state(gpio_events::Inputs::Mag2S) == Level::High,
    //         mag_1_n: button_states.get_state(gpio_events::Inputs::Mag1N) == Level::High,
    //         mag_1_s: button_states.get_state(gpio_events::Inputs::Mag1S) == Level::High,
    //         magazine_type: magazine_type,
    //         usb_vbus_fault: button_states.get_state(gpio_events::Inputs::UsbVbusFault)
    //             == Level::High,
    //         charger_status: button_states.get_state(gpio_events::Inputs::ChargerStatus)
    //             == Level::High,
    //         vbus_sns: button_states.get_state(gpio_events::Inputs::VbusSns) == Level::High,
    //         // HV sense data
    //         cart_1_counts: hv_sense.cart_1_counts,
    //         cart_1_state: hv_sense.cart_1_state,
    //         cart_2_counts: hv_sense.cart_2_counts,
    //         cart_2_state: hv_sense.cart_2_state,
    //         cart_3_counts: hv_sense.cart_3_counts,
    //         cart_3_state: hv_sense.cart_3_state,
    //         cart_4_counts: hv_sense.cart_4_counts,
    //         cart_4_state: hv_sense.cart_4_state,
    //         p_cap_sns_counts: hv_sense.p_cap_sns_counts,
    //         hv_pos_volts: hv_sense.hv_pos_volts,
    //         p_msc_sns_counts: hv_sense.p_msc_sns_counts,
    //         p_msc_volts: hv_sense.p_msc_volts,
    //         n_msc_sns_counts: hv_sense.n_msc_sns_counts,
    //         n_msc_volts: hv_sense.n_msc_volts,
    //         pbat_sns_counts: hv_sense.pbat_sns_counts,
    //         pbat_sns_mv: hv_sense.pbat_sns_mv,
    //         pbias_sns_counts: hv_sense.pbias_sns_counts,
    //         pbias_sns_mv: hv_sense.pbias_sns_mv,
    //         p3p3v_sns_counts: hv_sense.p3p3v_sns_counts,
    //         p3p3v_sns_mv: hv_sense.p3p3v_sns_mv,
    //         // Battery gauge data
    //         battery_voltage_mv: self.shared_sensor_data.get_battery_voltage_mv(),
    //         battery_soc_percent: self.shared_sensor_data.get_battery_soc_percent(),
    //         battery_current_ma: self.shared_sensor_data.get_battery_current_ma(),
    //         battery_temperature_c: self.shared_sensor_data.get_battery_temperature_c(),
    //         battery_remaining_mah: self.shared_sensor_data.get_battery_remaining_mah(),
    //         battery_valid: self.shared_sensor_data.is_battery_valid(),
    //     };
    //     let rsp = Response::GetSense(rsp);
    //     self.set_rsp(rsp);
    // }

    // async fn get_version(&mut self, bank: Option<i8>) {
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;

    //     let hv_rsp = match bank {
    //         Some(bank_id) => locked_hv_api
    //             .get_bank_version(bank_id)
    //             .await
    //             .unwrap_or_else(|_| thunderbird_types::API_rsp_version {
    //                 bank: -1,
    //                 build: cumulus_bootloader_types::build {
    //                     version: cumulus_bootloader_types::version {
    //                         major: -1,
    //                         minor: -1,
    //                         patch: -1,
    //                     },
    //                     build_time: -1,
    //                     version_string: [0u8; 10],
    //                 },
    //                 status: cumulus_bootloader_types::APHD_bank_status {
    //                     verified: cumulus_bootloader_types::APHD_verified::not_verified,
    //                     rejected: cumulus_bootloader_types::APHD_rejected::rejected,
    //                     tested: cumulus_bootloader_types::APHD_tested::not_tested,
    //                     reserved_2: 0x1FFF_FFFF_FFFF_FFFF,
    //                 },
    //             }),
    //         None => locked_hv_api.get_version().await.unwrap_or_else(|_| {
    //             thunderbird_types::API_rsp_version {
    //                 bank: -1,
    //                 build: cumulus_bootloader_types::build {
    //                     version: cumulus_bootloader_types::version {
    //                         major: -1,
    //                         minor: -1,
    //                         patch: -1,
    //                     },
    //                     build_time: -1,
    //                     version_string: [0u8; 10],
    //                 },
    //                 status: cumulus_bootloader_types::APHD_bank_status {
    //                     verified: cumulus_bootloader_types::APHD_verified::not_verified,
    //                     rejected: cumulus_bootloader_types::APHD_rejected::rejected,
    //                     tested: cumulus_bootloader_types::APHD_tested::not_tested,
    //                     reserved_2: 0x1FFF_FFFF_FFFF_FFFF,
    //                 },
    //             }
    //         }),
    //     };

    //     let bank = bank.unwrap_or(Self::get_this_bank());

    //     let main_status_address = match bank {
    //         0 => memory::BOOTLOADER_STATUS,
    //         1 => memory::BANK1_STATUS,
    //         2 => memory::BANK2_STATUS,
    //         _ => {
    //             error!("Invalid Bank: Bank {}", bank);
    //             return;
    //         }
    //     };
    //     let main_status = main_status_address as *const u64;
    //     let main_status = unsafe { core::ptr::read_unaligned(main_status) };
    //     let main_status_bytes = main_status.to_le_bytes();
    //     let main_status = match ion_types::APHD_bank_status::try_from(main_status_bytes) {
    //         Ok(status) => status,
    //         Err(e) => {
    //             error!(
    //                 "Failed to convert main status: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //             return;
    //         }
    //     };
    //     let main_header_address = match bank {
    //         0 => BOOTLOADER_BUILD,
    //         1 => BANK1_BUILD,
    //         2 => BANK2_BUILD,
    //         _ => {
    //             return;
    //         }
    //     };

    //     let header_ptr = main_header_address as *const cumulus_bootloader_types::build_slice;
    //     let build_slice: cumulus_bootloader_types::build_slice =
    //         unsafe { core::ptr::read_unaligned(header_ptr) };
    //     // info!("{:?}", build_slice);
    //     let main_build: cumulus_bootloader_types::build = match build_slice.try_into() {
    //         Ok(build) => build,
    //         Err(e) => {
    //             error!(
    //                 "Failed to convert build slice: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //             return;
    //         }
    //     };
    //     let rsp = Response::BankVersion(ion_types::BankVersionRsp {
    //         main_bank: bank,
    //         main_build,
    //         main_status,
    //         hv_bank: hv_rsp.bank,
    //         hv_build: hv_rsp.build,
    //         hv_status: hv_rsp.status,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn soft_reset(&mut self) {
    //     let rsp = Response::Reset(ion_types::ResetRsp {});
    //     self.set_rsp(rsp);
    //     self.send().await;
    //     info!("SOFT reset");

    //     {
    //         let mut stats = self.lifetime_stats.lock().await;
    //         stats.stats.resets = stats.stats.resets.saturating_add(1);
    //         if let Err(e) = stats.write().await {
    //             error!(
    //                 "Failed to persist lifetime_stats on reset: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //         }
    //     }

    //     let mut locked_device_pwr_en = self.device_pwr_en_mutex.lock().await;
    //     locked_device_pwr_en.set_low();
    //     Timer::after(Duration::from_millis(10)).await;
    //     SCB::sys_reset(); // NO RETURN
    // }

    // fn on_chip_flash_write(&mut self) {
    //     let (address, data) = match &*self.command {
    //         Command::OnChipFlashWrite(opts) => (opts.address, opts.data),
    //         _ => {
    //             error!("on_chip_flash_write called but current command is not OnChipFlashWrite");
    //             return;
    //         }
    //     };

    //     // Embassy flash API takes offsets from FLASH_BASE, not absolute addresses.
    //     let offset = (address - memory::FLASH_BASE as i32) as u32;
    //     const PAGE_SIZE: u32 = 8 * 1024;

    //     if let Err(e) = self
    //         .on_chip_flash
    //         .blocking_erase(offset, offset + PAGE_SIZE)
    //     {
    //         error!("Failed to erase flash: {:?}", e);
    //         return;
    //     }
    //     match self.on_chip_flash.blocking_write(offset, &data[..]) {
    //         Ok(_) => {
    //             let rsp = Response::FlashWrite(ion_types::FlashWriteRsp { address });
    //             self.set_rsp(rsp);
    //             info!("Wrote to {:#010X}", address);
    //         }
    //         Err(e) => {
    //             error!("Failed to write to flash: {:?}", e);
    //         }
    //     }
    // }

    // fn on_chip_flash_read(&mut self) {
    //     let address = match &*self.command {
    //         Command::OnChipFlashRead(opts) => opts.address,
    //         _ => {
    //             error!("on_chip_flash_read called but current command is not OnChipFlashRead");
    //             return;
    //         }
    //     };
    //     let mut data = [0u8; 256];

    //     // Embassy flash API takes offsets from FLASH_BASE, not absolute addresses.
    //     let read_result = self
    //         .on_chip_flash
    //         .blocking_read(address - memory::FLASH_BASE as u32, &mut data);
    //     match read_result {
    //         Ok(()) => {
    //             info!("Flash read successful!");
    //         }
    //         Err(Error::Size) => {
    //             error!("Flash read error: Data read out of bounds.");
    //         }
    //         Err(e) => {
    //             error!("Flash read error: {:?}", e);
    //         }
    //     }
    //     let rsp = Response::FlashRead(ion_types::FlashReadRsp { data });
    //     self.set_rsp(rsp);
    // }

    // async fn hv_flash_write(&mut self) {
    //     let opts = match &*self.command {
    //         Command::HvFlashWrite(opts) => opts,
    //         _ => {
    //             error!("hv_flash_write called but current command is not HvFlashWrite");
    //             return;
    //         }
    //     };
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     let address = match locked_hv_api
    //         .write_flash_page(opts.address, &opts.data)
    //         .await
    //     {
    //         Ok(address) => address,
    //         Err(_e) => {
    //             error!("Failed to write to flash");
    //             self.set_rsp(Response::FlashWrite(ion_types::FlashWriteRsp {
    //                 address: -1,
    //             }));
    //             return;
    //         }
    //     };

    //     let rsp = Response::FlashWrite(ion_types::FlashWriteRsp { address });
    //     self.set_rsp(rsp);
    // }

    // async fn buzzer_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::BuzzerSet(opts) => opts,
    //         _ => {
    //             error!("buzzer_set called but current command is not BuzzerSet");
    //             return;
    //         }
    //     };
    //     let mut locked_audio_pwm = self.audio_pwm_mutex.lock().await;
    //     locked_audio_pwm.set_frequency(time::hz(opts.frequency.try_into().unwrap_or(2700)));
    //     let duty_cycle_percent: u16 = {
    //         let max_duty = locked_audio_pwm.get_max_duty() as f32;
    //         let percent = opts.duty_cycle_percent as f32;
    //         let value = max_duty * percent / 100.0;
    //         value as u16
    //     };

    //     if duty_cycle_percent > 1 {
    //         locked_audio_pwm.set_duty(embassy_stm32::lptim::Channel::Ch1, duty_cycle_percent);
    //         locked_audio_pwm.enable(embassy_stm32::lptim::Channel::Ch1);
    //     } else {
    //         locked_audio_pwm.disable(embassy_stm32::lptim::Channel::Ch1);
    //     }

    //     let rsp = Response::BuzzerSet(ion_types::BuzzerSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn hv_reset(&mut self) {
    //     {
    //         let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //         let _ = locked_hv_api.reset().await.unwrap_or_else(|_| ());
    //     }

    //     let rsp = Response::Reset(ion_types::ResetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn device_power_set(&mut self) {
    //     let mode = match &*self.command {
    //         Command::DevicePower(opts) => opts.mode,
    //         _ => {
    //             error!("device_power_set called but current command is not DevicePower");
    //             return;
    //         }
    //     };
    //     use crate::power::{awake, sleep};
    //     match mode {
    //         ion_types::PowerMode::On => {
    //             awake(self.weapon_state_publisher, self.device_pwr_en_mutex).await
    //         }
    //         ion_types::PowerMode::Off => {
    //             sleep(self.weapon_state_publisher, self.device_pwr_en_mutex).await
    //         }
    //     }
    //     let rsp = Response::DevicePower(ion_types::DevicePowerRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn force_fire_cartridge(&mut self) {
    //     let opts = match &*self.command {
    //         Command::CartridgeForceFire(opts) => opts,
    //         _ => {
    //             error!("force_fire_cartridge called but current command is not CartridgeForceFire");
    //             return;
    //         }
    //     };
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     let _ = locked_hv_api
    //         .force_fire_cartridge(opts.cartridge)
    //         .await
    //         .unwrap_or_else(|_| ());
    //     let rsp = Response::CartridgeForceFire(ion_types::CartridgeForceFireRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn nmi_start(&mut self) {
    //     let opts = match &*self.command {
    //         Command::NmiStart(opts) => opts.clone(),
    //         _ => {
    //             error!("nmi_start called but current command is not NmiStart");
    //             return;
    //         }
    //     };
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     if let Err(e) = locked_hv_api
    //         .nmi_start(
    //             opts.high_cartridge,
    //             opts.low_cartridge,
    //             opts.pulse_rate,
    //             opts.pulse_count,
    //         )
    //         .await
    //     {
    //         error!("NmiStart failed: {:?}", defmt::Debug2Format(&e));
    //     }
    //     let rsp = Response::NmiStart(ion_types::NmiStartRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn flashlight_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::FlashlightSet(opts) => opts,
    //         _ => {
    //             error!("flashlight_set called but current command is not FlashlightSet");
    //             return;
    //         }
    //     };
    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::FlashlightPwmSet {
    //             frequency: opts.frequency,
    //             duty_cycle_percent: opts.duty_cycle_percent as u8,
    //         });
    //     let rsp = Response::FlashlightSet(ion_types::FlashlightSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn led_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::LedSet(opts) => opts,
    //         _ => {
    //             error!("led_set called but current command is not LedSet");
    //             return;
    //         }
    //     };
    //     let duty = opts.duty_cycle_percent.clamp(0, 100) as u8;

    //     match (opts.led_type, opts.color) {
    //         (ion_types::LedType::Battery, ion_types::LedColor::Off) => {
    //             self.battery_led_hw.set_all_colors_off().await;
    //         }
    //         (ion_types::LedType::Battery, ion_types::LedColor::Red) => {
    //             self.battery_led_hw
    //                 .set_red_pwm_duty_cycle_percent(duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::Battery, ion_types::LedColor::Green) => {
    //             self.battery_led_hw
    //                 .set_green_pwm_duty_cycle_percent(duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::Battery, ion_types::LedColor::Yellow) => {
    //             self.battery_led_hw
    //                 .set_yellow_pwm_duty_cycle_percent(duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::BLE, ion_types::LedColor::Off) => {
    //             self.ble_led_hw.set_all_colors_off().await;
    //         }
    //         (ion_types::LedType::BLE, ion_types::LedColor::Blue) => {
    //             self.ble_led_hw.set_blue_pwm_duty_cycle_percent(duty).await;
    //         }
    //         (ion_types::LedType::Cartridge_1, color) => {
    //             self.set_cartridge_led(thunderbird_types::LED::cartridge_1, color, duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::Cartridge_2, color) => {
    //             self.set_cartridge_led(thunderbird_types::LED::cartridge_2, color, duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::Cartridge_3, color) => {
    //             self.set_cartridge_led(thunderbird_types::LED::cartridge_3, color, duty)
    //                 .await;
    //         }
    //         (ion_types::LedType::Cartridge_4, color) => {
    //             self.set_cartridge_led(thunderbird_types::LED::cartridge_4, color, duty)
    //                 .await;
    //         }
    //         (led_type, color) => {
    //             error!(
    //                 "Invalid LED request: type={} color={}",
    //                 led_type as u8, color as u8
    //             );
    //         }
    //     }

    //     let rsp = Response::LedSet(ion_types::LedSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn set_cartridge_led(
    //     &mut self,
    //     cartridge: thunderbird_types::LED,
    //     color: ion_types::LedColor,
    //     duty: u8,
    // ) {
    //     let led_color = match color {
    //         ion_types::LedColor::Off => thunderbird_types::LED_color {
    //             red_percent: 0,
    //             green_percent: 0,
    //             blue_percent: 0,
    //         },
    //         ion_types::LedColor::Red => thunderbird_types::LED_color {
    //             red_percent: duty,
    //             green_percent: 0,
    //             blue_percent: 0,
    //         },
    //         ion_types::LedColor::Green => thunderbird_types::LED_color {
    //             red_percent: 0,
    //             green_percent: duty,
    //             blue_percent: 0,
    //         },
    //         ion_types::LedColor::Blue => thunderbird_types::LED_color {
    //             red_percent: 0,
    //             green_percent: 0,
    //             blue_percent: duty,
    //         },
    //         ion_types::LedColor::Yellow => thunderbird_types::LED_color {
    //             red_percent: duty,
    //             green_percent: duty,
    //             blue_percent: 0,
    //         },
    //     };

    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     if let Err(e) = locked_hv_api.set_cartridge_led(cartridge, led_color).await {
    //         error!("Failed to set cartridge LED: {:?}", defmt::Debug2Format(&e));
    //     }
    // }

    // async fn laser_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::LaserSet(opts) => opts,
    //         _ => {
    //             error!("laser_set called but current command is not LaserSet");
    //             return;
    //         }
    //     };
    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::LaserSet { state: opts.state });

    //     let rsp = Response::LaserSet(ion_types::LaserSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn laser_get_calibration_table(&mut self) {
    //     let mut buf = [0u8; LASER_CALIBRATION_SIZE];

    //     let mut eeprom_lock = self.shared_eeprom.lock().await;
    //     let mut address = crate::eeprom_map::LASER_CALIBRATION_ADDR;

    //     match eeprom_lock.read(address, &mut buf).await {
    //         Ok(_) => {
    //             info!("Read laser calibration table from EEPROM");
    //         }
    //         Err(e) => {
    //             error!(
    //                 "Failed to read laser calibration table from EEPROM: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //             return;
    //         }
    //     }
    //     drop(eeprom_lock);

    //     let calibration_table: LSR_calibration_table = match buf.as_slice().try_into() {
    //         Ok(table) => table,
    //         Err(_) => {
    //             error!("Failed to deserialize laser calibration table from EEPROM");
    //             return;
    //         }
    //     };

    //     let rsp = Response::LaserGetCalibrationTable(ion_types::LaserGetCalibrationTableRsp {
    //         laser_calibration_table: calibration_table,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn laser_set_calibration_table(&mut self) {
    //     let opts = match &*self.command {
    //         Command::LaserSetCalibrationTable(opts) => opts,
    //         _ => {
    //             error!(
    //                 "laser_set_calibration_table called but current command is not LaserSetCalibrationTable"
    //             );
    //             return;
    //         }
    //     };

    //     let calibration_table = opts.laser_calibration_table.clone();
    //     let mut bytes: [u8; LASER_CALIBRATION_SIZE] = calibration_table.clone().into();

    //     if let Err(e) = crc16::insert(&mut bytes) {
    //         error!(
    //             "Failed to compute CRC for calibration table: {:?}",
    //             defmt::Debug2Format(&e)
    //         );
    //         return;
    //     }

    //     let mut eeprom_lock = self.shared_eeprom.lock().await;
    //     let mut address = crate::eeprom_map::LASER_CALIBRATION_ADDR;

    //     // TODO: add write() to eeprom driver
    //     for chunk in bytes.chunks(16) {
    //         let mut page = [0u8; 16];
    //         page[..chunk.len()].copy_from_slice(chunk);
    //         match eeprom_lock.write_page(&page, address).await {
    //             Ok(_) => {
    //                 info!("Wrote laser calibration page at {:#06X}", address);
    //             }
    //             Err(e) => {
    //                 error!(
    //                     "Failed to write laser calibration to EEPROM: {:?}",
    //                     defmt::Debug2Format(&e)
    //                 );
    //                 return;
    //             }
    //         }
    //         address += 16;
    //     }
    //     drop(eeprom_lock);

    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::LaserSetCalibrationTable {
    //             calibration_table: calibration_table.clone(),
    //         });
    //     let rsp = Response::LaserSetCalibrationTable(ion_types::LaserSetCalibrationTableRsp {
    //         laser_calibration_table: calibration_table,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn time_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::TimeSet(opts) => opts,
    //         _ => {
    //             error!("time_set called but current command is not TimeSet");
    //             return;
    //         }
    //     };

    //     // Set local RTC (epoch is in seconds, rtc crate expects milliseconds)
    //     let epoch_ms = (opts.linux_epoch as u64) * 1000;
    //     match rtc::set_time(epoch_ms) {
    //         Ok(_) => {
    //             info!("RTC set to epoch {}", opts.linux_epoch);
    //         }
    //         Err(e) => {
    //             error!("Failed to set RTC: {}", e);
    //         }
    //     }

    //     // Also forward to HV board
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     match locked_hv_api.time_set(opts.linux_epoch, 0).await {
    //         Ok(_) => {}
    //         Err(e) => {
    //             error!("Failed to set HV RTC: {:?}", defmt::Debug2Format(&e));
    //         }
    //     }

    //     let rsp = Response::TimeSet(ion_types::TimeSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn time_get(&mut self) {
    //     // Read from local RTC first, fall back to HV board
    //     let epoch = if let Some(epoch_ms) = rtc::get_timestamp_ms() {
    //         (epoch_ms / 1000) as i64
    //     } else {
    //         let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //         match locked_hv_api.time_get().await {
    //             Ok(epoch) => epoch,
    //             Err(e) => {
    //                 error!("Failed to get RTC: {:?}", defmt::Debug2Format(&e));
    //                 0
    //             }
    //         }
    //     };
    //     let rsp = Response::TimeGet(ion_types::TimeGetRsp { linux_epoch: epoch });
    //     self.set_rsp(rsp);
    // }

    // fn get_gauge_info(&mut self) {
    //     let sd = self.shared_sensor_data;
    //     let rsp = Response::GetGaugeInfo(ion_types::GetGaugeInfoRsp {
    //         valid: sd.is_battery_valid(),
    //         voltage_mv: sd.get_battery_voltage_mv(),
    //         soc_percent: sd.get_battery_soc_percent(),
    //         current_ma: sd.get_battery_current_ma(),
    //         temperature_c: sd.get_battery_temperature_c(),
    //         remaining_mah: sd.get_battery_remaining_mah(),
    //         full_charge_capacity_mah: sd.get_battery_full_charge_capacity_mah(),
    //         average_power_mw: sd.get_battery_average_power_mw(),
    //         state_of_health_percent: sd.get_battery_state_of_health_percent(),
    //         battery_detected: sd.is_battery_detected(),
    //         charging: sd.is_battery_charging(),
    //         discharging: sd.is_battery_discharging(),
    //         full_charged: sd.is_battery_full_charged(),
    //         firmware_version: sd.get_gauge_firmware_version(),
    //         chemistry_id: sd.get_gauge_chemistry_id(),
    //     });
    //     self.set_rsp(rsp);
    // }

    // fn set_rsp(&mut self, rsp: Response) {
    //     let size = rsp.size();
    //     let rsp_slice: ion_types::Response_slice = rsp.into();

    //     self.rsp_buf[..size].copy_from_slice(&rsp_slice[..size]);
    //     self.rsp_len = size;
    // }

    fn flush_cmd_buf(&mut self) {
        info!("Flush cmd buffer");
        self.rec_len = 0;
        // self.rsp_len = 0;
        self.rec_stale = Instant::MAX;
    }

    // fn flush_send_buf(&mut self) {
    //     info!("Flush send buffer");
    //     self.rsp_len = 0;
    //     self.rsp_buf.fill(0);
    // }

    // async fn eeprom_write(&mut self) {
    //     let opts = match &*self.command {
    //         Command::EepromWrite(opts) => opts,
    //         _ => {
    //             error!("eeprom_write called but current command is not EepromWrite");
    //             return;
    //         }
    //     };
    //     let mut eeprom_lock = self.shared_eeprom.lock().await;

    //     match eeprom_lock
    //         .write_page(&opts.data, u16::from_be_bytes(opts.address))
    //         .await
    //     {
    //         Ok(_) => {
    //             let rsp = Response::EepromWrite(ion_types::EepromWriteRsp {});
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => {
    //             error!("Failed to write to EEPROM: {:?}", defmt::Debug2Format(&e));
    //         }
    //     }
    // }

    // async fn eeprom_read(&mut self) {
    //     let opts = match &*self.command {
    //         Command::EepromRead(opts) => opts,
    //         _ => {
    //             error!("eeprom_read called but current command is not EepromRead");
    //             return;
    //         }
    //     };
    //     let mut data = [0u8; 16];
    //     let mut eeprom_lock = self.shared_eeprom.lock().await;
    //     match eeprom_lock
    //         .read(u16::from_be_bytes(opts.address), &mut data)
    //         .await
    //     {
    //         Ok(_) => {
    //             let rsp = Response::EepromRead(ion_types::EepromReadRsp { page: data });
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => {
    //             error!("Failed to read from EEPROM: {:?}", defmt::Debug2Format(&e));
    //         }
    //     }
    // }

    // async fn product_info_get(&mut self) {
    //     let product_info = self.product_info.lock().await;
    //     let rsp = Response::ProductInfoGet(ion_types::ProductInfoGetRsp {
    //         product_info: product_info.info.clone(),
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn lifetime_stats_get(&mut self) {
    //     let lifetime_stats = self.lifetime_stats.lock().await;
    //     let rsp = Response::LifetimeStatsGet(ion_types::LifetimeStatsGetRsp {
    //         lifetime_stats: lifetime_stats.stats.clone(),
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn product_info_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::ProductInfoSet(opts) => opts,
    //         _ => {
    //             error!("product_info_set called but current command is not ProductInfoSet");
    //             return;
    //         }
    //     };
    //     let mut product_info = self.product_info.lock().await;
    //     product_info.info = opts.product_info.clone();
    //     match product_info.write().await {
    //         Ok(_) => {
    //             let rsp = Response::ProductInfoSet(ion_types::ProductInfoSetRsp {});
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => error!(
    //             "Failed to write product info: {:?}",
    //             defmt::Debug2Format(&e)
    //         ),
    //     }
    // }

    // async fn flashlight_strobe_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::FlashlightStrobeSet(opts) => opts,
    //         _ => {
    //             error!(
    //                 "flashlight_strobe_set called but current command is not FlashlightStrobeSet"
    //             );
    //             return;
    //         }
    //     };
    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::FlashlightStrobeSet {
    //             strobe_frequency: opts.strobe_frequency,
    //             strobe_on_duty_cycle_percent: opts.on_duty_cycle_percent,
    //             strobe_off_duty_cycle_percent: opts.off_duty_cycle_percent,
    //         });
    //     let rsp = Response::FlashlightStrobeSet(ion_types::FlashlightStrobeSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn buzzer_profile_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::BuzzerProfileSet(opts) => opts,
    //         _ => {
    //             error!("buzzer_profile_set called but current command is not BuzzerProfileSet");
    //             return;
    //         }
    //     };

    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::BuzzerProfileSet {
    //             profile: opts.profile,
    //         });

    //     let rsp = Response::BuzzerProfileSet(ion_types::BuzzerProfileSetRsp {});
    //     self.set_rsp(rsp);
    // }

    // async fn off_chip_flash_erase(&mut self) {
    //     let opts = match &*self.command {
    //         Command::OffChipFlashErase(opts) => opts,
    //         _ => {
    //             error!("off_chip_flash_erase called but current command is not OffChipFlashErase");
    //             return;
    //         }
    //     };
    //     let mut flash_lock = self.shared_flash.lock().await;

    //     match flash_lock
    //         .erase(is25lp128f::FlashEraseSize::EraseSize4K, opts.address)
    //         .await
    //     {
    //         Ok(_) => {
    //             let rsp = Response::OffChipFlashErase(ion_types::OffChipFlashEraseRsp {});
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => {
    //             error!(
    //                 "Failed to erase off-chip flash: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //         }
    //     }
    // }

    // async fn off_chip_flash_write(&mut self) {
    //     let opts = match &*self.command {
    //         Command::OffChipFlashWrite(opts) => opts,
    //         _ => {
    //             error!("off_chip_flash_write called but current command is not OffChipFlashWrite");
    //             return;
    //         }
    //     };
    //     let mut flash_lock = self.shared_flash.lock().await;

    //     match flash_lock.write_page(opts.address, &opts.data).await {
    //         Ok(_) => {
    //             info!(
    //                 "Flash write OK {} bytes @0x{:08x}",
    //                 opts.data.len(),
    //                 opts.address
    //             );
    //             let rsp = Response::OffChipFlashWrite(ion_types::OffChipFlashWriteRsp {});
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => {
    //             error!(
    //                 "Failed to write to off-chip flash: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //         }
    //     }
    // }

    // async fn off_chip_flash_read(&mut self) {
    //     let opts = match &*self.command {
    //         Command::OffChipFlashRead(opts) => opts,
    //         _ => {
    //             error!("off_chip_flash_read called but current command is not OffChipFlashRead");
    //             return;
    //         }
    //     };
    //     let mut data = [0u8; 16];
    //     let mut flash_lock = self.shared_flash.lock().await;
    //     match flash_lock.read(opts.address, &mut data).await {
    //         Ok(_) => {
    //             let rsp = Response::OffChipFlashRead(ion_types::OffChipFlashReadRsp { page: data });
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => {
    //             error!(
    //                 "Failed to read from off-chip flash: {:?}",
    //                 defmt::Debug2Format(&e)
    //             );
    //         }
    //     }
    // }

    // async fn off_chip_flash_read_product_id(&mut self) {
    //     let opts = match &*self.command {
    //         Command::OffChipFlashReadProductId(opts) => opts,
    //         _ => {
    //             error!(
    //                 "off_chip_flash_read_product_id called but current command is not OffChipFlashReadProductId"
    //             );
    //             return;
    //         }
    //     };

    //     let mut flash_lock = self.shared_flash.lock().await;

    //     let flash_jedec_id = match flash_lock.read_jedec_id().await {
    //         Ok(jedec_id) => jedec_id,
    //         Err(err) => {
    //             error!("Failed to read JEDEC_ID from flash.");
    //             return;
    //         }
    //     };
    //     let rsp = Response::OffChipFlashReadProductId(ion_types::OffChipFlashReadProductIdRsp {
    //         product_id: [flash_jedec_id.mfr_code],
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn read_unique_ids(&mut self) {
    //     // Read off-chip flash unique ID
    //     let off_chip_flash_uid = {
    //         let mut flash_lock = self.shared_flash.lock().await;
    //         match flash_lock.read_unique_id().await {
    //             Ok(uid) => uid,
    //             Err(_) => {
    //                 error!("Failed to read off-chip flash unique ID");
    //                 [0xFF; 16]
    //             }
    //         }
    //     };

    //     // Read STM32WBA65 96-bit unique ID
    //     let mcu_uid = uid::uid();

    //     // Flash/RAM size and package from device electronic signature registers
    //     let flashsizer = DESIG.flashsizer().read();
    //     let flash_size_kb: u16 = flashsizer.flash_size();
    //     let ram_size_kb: u16 = flashsizer.ram_size();
    //     let pkgr = DESIG.pkgr().read();
    //     let package: u8 = pkgr.pkg().to_bits();

    //     // IEEE 64-bit UID components
    //     let uid64r1 = DESIG.uid64r1().read();
    //     let uid64r2 = DESIG.uid64r2().read();
    //     let uid64_devnum: u32 = uid64r1.devnum();
    //     let uid64_devid: u8 = uid64r2.devid().to_bits();
    //     let uid64_stid: u32 = uid64r2.stid().to_bits();

    //     let rsp = Response::UniqueIdsRead(ion_types::UniqueIdsReadRsp {
    //         off_chip_flash_uid,
    //         mcu_uid: *mcu_uid,
    //         flash_size: flash_size_kb,
    //         ram_size: ram_size_kb,
    //         package,
    //         uid64_devnum,
    //         uid64_devid,
    //         uid64_stid,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn logs_get(&mut self) {
    //     let (partition, start_index) = match &*self.command {
    //         Command::LogsGet(opts) => (opts.partition as u8, opts.start_index as u32),
    //         _ => {
    //             error!("logs_get called but current command is not LogsGet");
    //             return;
    //         }
    //     };

    //     let buf = match self.logs_read_buf.take() {
    //         Some(buf) => buf,
    //         None => {
    //             error!("logs_read_buf unavailable (previous LogsGet leaked ownership)");
    //             return;
    //         }
    //     };

    //     self.logs_read_reply.reset();
    //     LOG_CTRL
    //         .send(LoggerCtrl::Read(LogReadReq {
    //             partition,
    //             start_index,
    //             out: buf,
    //             reply: self.logs_read_reply,
    //         }))
    //         .await;
    //     let LogReadRsp {
    //         size,
    //         count,
    //         err,
    //         out,
    //     } = self.logs_read_reply.wait().await;

    //     if let Some(e) = err {
    //         error!("LogsGet read failed: {:?}", defmt::Debug2Format(&e));
    //     }

    //     let mut data = [0u8; LOGS_READ_BUF_SIZE];
    //     data[..size].copy_from_slice(&out[..size]);

    //     let restored: &'static mut [u8; LOGS_READ_BUF_SIZE] = out
    //         .try_into()
    //         .expect("logs_read reply buffer length mismatch");
    //     self.logs_read_buf = Some(restored);

    //     let rsp = Response::LogsGet(LogsGetRsp {
    //         count: count as i16,
    //         size: size as i16,
    //         data,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn logs_erase(&mut self) {
    //     let opts = match &*self.command {
    //         Command::LogsErase(opts) => opts,
    //         _ => {
    //             error!("logs_erase called but current command is not LogsErase");
    //             return;
    //         }
    //     };

    //     self.logs_erase_reply.reset();
    //     LOG_CTRL
    //         .send(LoggerCtrl::Erase(LogEraseReq {
    //             partition: opts.partition as u8,
    //             reply: self.logs_erase_reply,
    //         }))
    //         .await;
    //     if let Err(e) = self.logs_erase_reply.wait().await {
    //         error!("LogsErase failed: {:?}", defmt::Debug2Format(&e));
    //     }

    //     self.set_rsp(Response::LogsErase(LogsEraseRsp {}));
    // }

    // async fn logs_flush(&mut self) {
    //     self.logs_flush_reply.reset();
    //     LOG_CTRL
    //         .send(LoggerCtrl::Flush(LogFlushReq {
    //             reply: self.logs_flush_reply,
    //         }))
    //         .await;
    //     if let Err(e) = self.logs_flush_reply.wait().await {
    //         error!("LogsFlush failed: {:?}", defmt::Debug2Format(&e));
    //     }

    //     self.set_rsp(Response::LogsFlush(LogsFlushRsp {}));
    // }

    // fn read_flash_rdp_option(&mut self) {
    //     let optr_rdp = pac::FLASH.optr().read().rdp();
    //     info!("API: FLASH-DRP = {:?}", optr_rdp);

    //     let rsp = Response::FlashRdpOptionRead(ion_types::FlashRdpOptionReadRsp {
    //         rdp: [u8::from(optr_rdp)],
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn ble_dtm_start_tx(&mut self) {
    //     let opts = match &*self.command {
    //         Command::BleDtmStartTx(opts) => opts,
    //         _ => {
    //             error!("ble_dtm_start_tx called but current command is not BleDtmStartTx");
    //             return;
    //         }
    //     };

    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::BleDtmStartTx {
    //             channel: opts.Channel,
    //             length: opts.Length,
    //             payload: opts.Payload,
    //         });

    //     let rsp = Response::BleDtmStartTx(ion_types::BleDtmStartTxRsp { Success: 1 });
    //     self.set_rsp(rsp);
    // }

    // async fn ble_dtm_start_rx(&mut self) {
    //     let opts = match &*self.command {
    //         Command::BleDtmStartRx(opts) => opts,
    //         _ => {
    //             error!("ble_dtm_start_rx called but current command is not BleDtmStartRx");
    //             return;
    //         }
    //     };

    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::BleDtmStartRx {
    //             channel: opts.Channel,
    //             length: opts.Length,
    //         });

    //     let rsp = Response::BleDtmStartRx(ion_types::BleDtmStartRxRsp { Success: 1 });
    //     self.set_rsp(rsp);
    // }

    // async fn ble_dtm_stop(&mut self) {
    //     match &*self.command {
    //         Command::BleDtmStop(_) => {}
    //         _ => {
    //             error!("ble_dtm_stop called but current command is not BleDtmStop");
    //             return;
    //         }
    //     };

    //     self.cli_event_publisher
    //         .publish_immediate(IonCliEvent::BleDtmStop {});

    //     let dtm_result = BLE_DTM_RESULT.receive().await;
    //     let rsp = Response::BleDtmStop(ion_types::BleDtmStopRsp {
    //         Success: i8::from(dtm_result.success),
    //         packet_count: dtm_result.packet_count as i16,
    //         duration_ms: dtm_result.duration_ms as i32,
    //         length: dtm_result.length as i8,
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn get_magazine_type(&mut self) -> ion_types::magazine_type {
    //     let magazine_lock = self.magazine_mutex.lock().await;
    //     magazine_lock.type_id
    // }

    // async fn device_settings_get(&mut self) {
    //     let settings = self.device_settings.lock().await;
    //     let rsp = Response::DeviceSettingsGet(ion_types::DeviceSettingsGetRsp {
    //         settings: settings.device_settings.clone(),
    //     });
    //     self.set_rsp(rsp);
    // }

    // async fn device_settings_set(&mut self) {
    //     let opts = match &*self.command {
    //         Command::DeviceSettingsSet(opts) => opts,
    //         _ => {
    //             error!("device_settings_set called but current command is not DeviceSettingsSet");
    //             return;
    //         }
    //     };
    //     let mut device_settings = self.device_settings.lock().await;
    //     device_settings.device_settings = opts.settings.clone();
    //     match device_settings
    //         .write(ion_types::DEV_settings_update_source::cli)
    //         .await
    //     {
    //         Ok(_) => {
    //             let rsp = Response::DeviceSettingsSet(ion_types::DeviceSettingsSetRsp {});
    //             self.set_rsp(rsp);
    //         }
    //         Err(e) => error!(
    //             "Failed to write device settings: {:?}",
    //             defmt::Debug2Format(&e)
    //         ),
    //     }
    // }

    // async fn hv_arm(&mut self) {
    //     let opts = match &*self.command {
    //         Command::HvArm(opts) => opts.clone(),
    //         _ => {
    //             error!("hv_arm called but current command is not HvArm");
    //             return;
    //         }
    //     };
    //     let mut locked_hv_api = self.hv_api_mutex.lock().await;
    //     if let Err(e) = locked_hv_api.arm(opts.arm).await {
    //         error!("HV arm failed: {:?}", defmt::Debug2Format(&e));
    //     }
    //     let rsp = Response::HvArm(ion_types::HvArmRsp {});
    //     self.set_rsp(rsp);
    // }
}
