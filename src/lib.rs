#![cfg_attr(not(test), no_std)]
use core::sync::atomic::AtomicUsize;
use cxx::private::c_char;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("mikanos-usb/external/error.hpp");
        type Error;
        fn Name(self: &Error) -> *const c_char;
        fn File(self: &Error) -> *const c_char;
        fn Line(self: &Error) -> i32;
    }
    unsafe extern "C++" {
        include!("test_cpp/test.hpp");
        fn new_success() -> UniquePtr<Error>;
    }

    #[namespace = "usb::xhci"]
    unsafe extern "C++" {
        include!("mikanos-usb/usb/xhci/xhci.hpp");
        type Controller;
        type Ring;
        type EventRing;
        type DoorbellRegister;
        type Port;
        type DeviceManager;
        // class Controller
        fn new_controller(mmio_base: usize) -> UniquePtr<Controller>;
        fn Initialize(self: Pin<&mut Controller>) -> UniquePtr<Error>;
        fn Run(self: Pin<&mut Controller>) -> UniquePtr<Error>;
        fn CommandRing(self: Pin<&mut Controller>) -> *mut Ring;
        fn PrimaryEventRing(self: Pin<&mut Controller>) -> *mut EventRing;
        fn DoorbellRegisterAt(self: Pin<&mut Controller>, index: u8) -> *mut DoorbellRegister;
        fn PortAt(self: Pin<&mut Controller>, port_num: u8) -> UniquePtr<Port>;
        fn MaxPorts(self: &Controller) -> u8;
        fn DeviceManager(self: Pin<&mut Controller>) -> *mut DeviceManager;
        // end class Controller

        // class Ring
        fn Initialize(self: Pin<&mut Ring>, buf_size: u64) -> UniquePtr<Error>;
        // end class Ring

        type TRB;
        type InterrupterRegisterSet;
        // class EventRing
        unsafe fn Initialize(
            self: Pin<&mut EventRing>,
            buf_size: u64,
            interrupter: *mut InterrupterRegisterSet,
        ) -> UniquePtr<Error>;
        fn ReadDequeuePointer(self: &EventRing) -> *mut TRB;
        unsafe fn WriteDequeuePointer(self: Pin<&mut EventRing>, p: *mut TRB);
        fn HasFront(self: &EventRing) -> bool;
        fn Front(self: &EventRing) -> *mut TRB;
        fn Pop(self: Pin<&mut EventRing>);
        // end class EventRing

        // class DoorbellRegister
        fn Ring(self: Pin<&mut DoorbellRegister>, target: u8, stream_id: u16);
        // end class DoorbellRegister

        type Device;
        // class Port
        fn Number(self: &Port) -> u8;
        fn IsConnected(self: &Port) -> bool;
        fn IsEnabled(self: &Port) -> bool;
        fn IsConnectStatusChanged(self: &Port) -> bool;
        fn IsPortResetChanged(self: &Port) -> bool;
        fn Speed(self: &Port) -> i32;
        fn Reset(self: Pin<&mut Port>) -> UniquePtr<Error>;
        fn Initialize(self: Pin<&mut Port>) -> *mut Device;
        fn ClearConnectStatusChanged(self: &Port);
        fn ClearPortResetChange(self: &Port);
        // end class Port

        type DeviceContext;
        // class DeviceManager
        fn Initialize(self: Pin<&mut DeviceManager>, max_slots: usize) -> UniquePtr<Error>;
        fn DeviceContexts(self: &DeviceManager) -> *mut *mut DeviceContext;
        fn FindByPort(self: &DeviceManager, port_num: u8, route_string: u32) -> *mut Device;
        // FindByState
        fn FindBySlot(self: &DeviceManager, slot_id: u8) -> *mut Device;
        unsafe fn AllocDevice(
            self: Pin<&mut DeviceManager>,
            slot_id: u8,
            dbreg: *mut DoorbellRegister,
        ) -> UniquePtr<Error>;
        fn LoadDCBAA(self: Pin<&mut DeviceManager>, slot_id: u8) -> UniquePtr<Error>;
        fn Remove(self: Pin<&mut DeviceManager>, slot_id: u8) -> UniquePtr<Error>;
        // end class DeviceManager
    }
}

#[cxx::bridge(namespace = "rust")]
mod rust {
    extern "Rust" {
        unsafe fn put_string(s: *const c_char) -> bool;
    }
}

type PrintFunc = fn(&str);
static LOG_PRINTER: AtomicUsize = AtomicUsize::new(0);

pub fn set_log_printer(printer: PrintFunc) {
    LOG_PRINTER.store(printer as usize, core::sync::atomic::Ordering::Release);
}

#[doc(hidden)]
unsafe fn put_string(s: *const c_char) -> bool {
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(s) }.to_str() else {return false};
    let printer = LOG_PRINTER.load(core::sync::atomic::Ordering::Acquire);
    if printer != 0 {
        let f: PrintFunc = unsafe { core::mem::transmute(printer) };
        f(s);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_test() {
        use core::ffi::CStr;
        let error = ffi::new_success();
        let name = unsafe { CStr::from_ptr(error.Name()) }.to_str().unwrap();
        let file = unsafe { CStr::from_ptr(error.File()) }.to_str().unwrap();
        let line = error.Line();
        assert_eq!(name, "kSuccess");
        assert_eq!(file, "test_cpp/test.cpp");
        assert_eq!(line, 6);
    }
}
