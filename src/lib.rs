mod bindings;

use std::{
    collections::VecDeque,
    convert::TryInto,
    os::raw::{c_char, c_short, c_uchar, c_ushort, c_void},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use bindings::*;

#[derive(Debug)]
pub enum Error {
    BusNotFound,
    NoFreeSlot,
    InvalidTarget,
    RemovalFailed,
    AlreadyConnected,
    TargetUninitialized,
    TargetNotPluggedIn,
    BusVersionMismatch,
    BusAccessFailed,
    CallbackAlreadyRegistered,
    CallbackNotFound,
    BusAlreadyConnected,
    BusInvalidHandle,
    XUSBUserIndexOutOfRange,
    InvalidParameter,
    NotSupported,
    PlugInError(Box<Error>, Box<Error>),
    Unknown,
}

pub trait ClientErrorConvertable {
    fn to_error(&self) -> Option<Error>;
}

impl ClientErrorConvertable for VIGEM_ERROR {
    fn to_error(&self) -> Option<Error> {
        match *self {
            _VIGEM_ERRORS_VIGEM_ERROR_NONE => None,
            _VIGEM_ERRORS_VIGEM_ERROR_BUS_NOT_FOUND => Some(Error::BusNotFound),
            _VIGEM_ERRORS_VIGEM_ERROR_NO_FREE_SLOT => Some(Error::NoFreeSlot),
            _VIGEM_ERRORS_VIGEM_ERROR_INVALID_TARGET => Some(Error::InvalidTarget),
            _VIGEM_ERRORS_VIGEM_ERROR_REMOVAL_FAILED => Some(Error::RemovalFailed),
            _VIGEM_ERRORS_VIGEM_ERROR_ALREADY_CONNECTED => Some(Error::AlreadyConnected),
            _VIGEM_ERRORS_VIGEM_ERROR_TARGET_UNINITIALIZED => Some(Error::TargetUninitialized),
            _VIGEM_ERRORS_VIGEM_ERROR_TARGET_NOT_PLUGGED_IN => Some(Error::TargetNotPluggedIn),
            _VIGEM_ERRORS_VIGEM_ERROR_BUS_VERSION_MISMATCH => Some(Error::BusVersionMismatch),
            _VIGEM_ERRORS_VIGEM_ERROR_BUS_ACCESS_FAILED => Some(Error::BusAccessFailed),
            _VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_ALREADY_REGISTERED => {
                Some(Error::CallbackAlreadyRegistered)
            }
            _VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_NOT_FOUND => Some(Error::CallbackNotFound),
            _VIGEM_ERRORS_VIGEM_ERROR_BUS_ALREADY_CONNECTED => Some(Error::BusAlreadyConnected),
            _VIGEM_ERRORS_VIGEM_ERROR_BUS_INVALID_HANDLE => Some(Error::BusInvalidHandle),
            _VIGEM_ERRORS_VIGEM_ERROR_XUSB_USERINDEX_OUT_OF_RANGE => {
                Some(Error::XUSBUserIndexOutOfRange)
            }
            _VIGEM_ERRORS_VIGEM_ERROR_INVALID_PARAMETER => Some(Error::InvalidParameter),
            _VIGEM_ERRORS_VIGEM_ERROR_NOT_SUPPORTED => Some(Error::NotSupported),
            _ => Some(Error::Unknown),
        }
    }
}

#[derive(Debug)]
pub enum Button {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    Start,
    Back,
    LeftThumb,
    RightThumb,
    LeftShoulder,
    RightShoulder,
    Guide,
    A,
    B,
    X,
    Y,
}

impl Into<XUSB_BUTTON> for Button {
    fn into(self) -> XUSB_BUTTON {
        match self {
            Button::DpadUp => _XUSB_BUTTON_XUSB_GAMEPAD_DPAD_UP,
            Button::DpadDown => _XUSB_BUTTON_XUSB_GAMEPAD_DPAD_DOWN,
            Button::DpadLeft => _XUSB_BUTTON_XUSB_GAMEPAD_DPAD_LEFT,
            Button::DpadRight => _XUSB_BUTTON_XUSB_GAMEPAD_DPAD_RIGHT,
            Button::Start => _XUSB_BUTTON_XUSB_GAMEPAD_START,
            Button::Back => _XUSB_BUTTON_XUSB_GAMEPAD_BACK,
            Button::LeftThumb => _XUSB_BUTTON_XUSB_GAMEPAD_LEFT_THUMB,
            Button::RightThumb => _XUSB_BUTTON_XUSB_GAMEPAD_RIGHT_THUMB,
            Button::LeftShoulder => _XUSB_BUTTON_XUSB_GAMEPAD_LEFT_SHOULDER,
            Button::RightShoulder => _XUSB_BUTTON_XUSB_GAMEPAD_RIGHT_SHOULDER,
            Button::Guide => _XUSB_BUTTON_XUSB_GAMEPAD_GUIDE,
            Button::A => _XUSB_BUTTON_XUSB_GAMEPAD_A,
            Button::B => _XUSB_BUTTON_XUSB_GAMEPAD_B,
            Button::X => _XUSB_BUTTON_XUSB_GAMEPAD_X,
            Button::Y => _XUSB_BUTTON_XUSB_GAMEPAD_Y,
        }
    }
}

#[derive(Debug)]
pub enum Input {
    Pressed(Button),
    Released(Button),
    PressedLeftTrigger(c_char),
    PressedRightTrigger(c_char),
    MovedLeftThumbStick(c_short, c_short),
    MovedRightThumbStick(c_short, c_short),
}

#[derive(Debug)]
pub enum Output {
    Rumble(c_uchar, c_uchar),
    Led(c_uchar),
}

pub struct Client {
    client: PVIGEM_CLIENT,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let client = unsafe { vigem_alloc() };

        unsafe {
            if let Some(error) = vigem_connect(client).to_error() {
                vigem_free(client);
                return Err(error);
            }
        }

        Ok(Self { client })
    }
}

pub trait ClientExt {
    fn plug_in(&mut self) -> Result<Device, Error>;
}

impl ClientExt for Arc<Mutex<Client>> {
    fn plug_in(&mut self) -> Result<Device, Error> {
        let target = unsafe { vigem_target_x360_alloc() };
        let client = self.lock().unwrap();

        let (raw_output_tx, raw_output_rx) = channel::<RawOutput>();

        unsafe {
            let mut raw_error = vigem_target_add(client.client, target);
            if let Some(error) = raw_error.to_error() {
                vigem_target_free(target);
                return Err(error);
            }

            raw_error = vigem_target_x360_register_notification(
                client.client,
                target,
                Some(x360_notification_callback),
                Box::into_raw(Box::new(raw_output_tx)) as *mut c_void,
            );
            if let Some(error) = raw_error.to_error() {
                raw_error = vigem_target_remove(client.client, target);

                vigem_target_free(target);

                if let Some(another_error) = raw_error.to_error() {
                    return Err(Error::PlugInError(Box::new(error), Box::new(another_error)));
                }
                return Err(error);
            }
        }

        Ok(Device::new(Arc::clone(&self), target, raw_output_rx))
    }
}

// we need to tell the compiler that the struct is safe to be
// sent across threads because we will use PVIGEM_CLIENT like
// an opaque pointer.
unsafe impl Send for Client {}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            vigem_disconnect(self.client);
            vigem_free(self.client);
        }
    }
}

struct RawOutput {
    large_motor: c_uchar,
    small_motor: c_uchar,
    led_number: c_uchar,
}

unsafe extern "C" fn x360_notification_callback(
    _client: PVIGEM_CLIENT,
    _target: PVIGEM_TARGET,
    large_motor: c_uchar,
    small_motor: c_uchar,
    led_number: c_uchar,
    user_data: *mut c_void,
) {
    // NOTE: do not touch the client and target here!!!

    let raw_output_tx_ptr = user_data as *mut Sender<RawOutput>;

    if let Err(e) = (*raw_output_tx_ptr).send(RawOutput {
        large_motor,
        small_motor,
        led_number,
    }) {
        log::error!(
            "Got an error sending the raw output from the ViGEm callback. Error: {}",
            e
        );
    }
}

struct DeviceNotificationLockGuard<'a> {
    target: &'a PVIGEM_TARGET,
}

impl<'a> DeviceNotificationLockGuard<'a> {
    fn new(target: &'a PVIGEM_TARGET) -> Self {
        unsafe {
            vigem_target_lock_notification(*target);
        }

        Self { target }
    }
}

impl<'a> Drop for DeviceNotificationLockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            vigem_target_unlock_notification(*self.target);
        }
    }
}

pub struct Device {
    client: Arc<Mutex<Client>>,
    raw_output_rx: Receiver<RawOutput>,
    latest_raw_output: RawOutput,
    output_queue: VecDeque<Output>,
    target: PVIGEM_TARGET,
    report: XUSB_REPORT,
}

impl Device {
    fn new(
        client: Arc<Mutex<Client>>,
        target: PVIGEM_TARGET,
        raw_output_rx: Receiver<RawOutput>,
    ) -> Self {
        Self {
            client,
            target,
            raw_output_rx,
            output_queue: VecDeque::new(),
            latest_raw_output: RawOutput {
                large_motor: 0,
                small_motor: 0,
                led_number: 0,
            },
            report: XUSB_REPORT {
                wButtons: 0,
                bLeftTrigger: 0,
                bRightTrigger: 0,
                sThumbLX: 0,
                sThumbLY: 0,
                sThumbRX: 0,
                sThumbRY: 0,
            },
        }
    }

    pub fn put_input(&mut self, input: Input) -> Result<(), Error> {
        match input {
            Input::Pressed(button) => {
                let raw_button: XUSB_BUTTON = button.into();
                let raw_button: c_ushort = raw_button.try_into().unwrap();
                self.report.wButtons |= raw_button;
            }
            Input::Released(button) => {
                let raw_button: XUSB_BUTTON = button.into();
                let raw_button: c_ushort = raw_button.try_into().unwrap();
                self.report.wButtons &= !raw_button;
            }
            Input::PressedLeftTrigger(level) => {
                self.report.bLeftTrigger = level;
            }
            Input::PressedRightTrigger(level) => {
                self.report.bRightTrigger = level;
            }
            Input::MovedLeftThumbStick(x, y) => {
                self.report.sThumbLX = x;
                self.report.sThumbLY = y;
            }
            Input::MovedRightThumbStick(x, y) => {
                self.report.sThumbRX = x;
                self.report.sThumbRY = y;
            }
        }

        let client = self.client.lock().unwrap();

        unsafe {
            let raw_error = vigem_target_x360_update(client.client, self.target, self.report);
            if let Some(error) = raw_error.to_error() {
                return Err(error);
            }
        }

        Ok(())
    }

    pub fn get_output(&mut self) -> Option<Output> {
        match self.raw_output_rx.try_recv() {
            Ok(raw_output) => {
                let did_large_motor_change =
                    raw_output.large_motor != self.latest_raw_output.large_motor;
                let did_small_motor_change =
                    raw_output.small_motor != self.latest_raw_output.small_motor;
                let did_led_number_change =
                    raw_output.led_number != self.latest_raw_output.led_number;
                let has_queued_output = self.output_queue.front().is_some();

                self.latest_raw_output = raw_output;

                let output = if did_large_motor_change || did_small_motor_change {
                    let rumble_output = Output::Rumble(
                        self.latest_raw_output.large_motor,
                        self.latest_raw_output.small_motor,
                    );
                    if has_queued_output {
                        // queue the rumble output for later
                        self.output_queue.push_back(rumble_output);
                        // get the latest queued output
                        self.output_queue.pop_front()
                    } else {
                        // we don't need to queue the rumble output since the queue is empty anyway
                        Some(rumble_output)
                    }
                } else if has_queued_output {
                    // there is no rumble output and the queue is not empty
                    self.output_queue.pop_front()
                } else {
                    // there is no rumble output and the queue is empty
                    None
                };

                if did_led_number_change {
                    let led_output = Output::Led(self.latest_raw_output.led_number);
                    if output.is_some() {
                        // queue the led output for later
                        self.output_queue.push_back(led_output);
                        // return the rumble output or the latest queued output here
                        output
                    } else {
                        // we don't need to queue the rumble output since the queue is empty
                        // and there is no rumble output
                        Some(led_output)
                    }
                } else {
                    // there is no led output so just return the rumble output
                    // or the latest queued output here
                    output
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // pop sumn from the output queue if there are no raw outputs available
                self.output_queue.pop_front()
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                log::error!("Tried to receive raw output but the other half of the channel (sender) is already closed.");
                None
            }
        }
    }

    pub fn unplug(self) -> Result<(), Error> {
        // we need to unplug the device explicitly because error/s may
        // occur here. drop will occur after this anyway since we are
        // going to consume self here.

        let client = self.client.lock().unwrap();
        let _guard = DeviceNotificationLockGuard::new(&self.target);

        unsafe {
            let user_data = vigem_target_x360_unregister_notification(self.target);

            Box::from_raw(user_data as *mut Sender<Output>);

            let raw_error = vigem_target_remove(client.client, self.target);
            if let Some(error) = raw_error.to_error() {
                return Err(error);
            }
        }

        Ok(())
    }
}

// we need to tell the compiler that the struct is safe to be
// sent across threads because we will use PVIGEM_TARGET like
// an opaque pointer.
unsafe impl Send for Device {}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            vigem_target_free(self.target);
        }
    }
}
