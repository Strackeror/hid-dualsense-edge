use std::os::raw::c_void;
use std::os::windows::raw::HANDLE;

use macro_rules_attribute::apply;
use paste::paste;
use widestring::{WideCStr, WideCString};

macro_rules! proxy {
    (fn $func:ident ($($name:ident : $arg:ty),*) -> $ret:ty {$($t:tt)*}) => {
        extern "C" {
            paste!{fn [<$func _orig>]($($name : $arg),*) -> $ret;}
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        extern "C" fn $func($($name : $arg),*) -> $ret {
            $($t)*
        }
    };
}

#[repr(C)]
#[derive(Debug)]
struct HidAttributes {
    _size: u32,
    vendor_id: u16,
    product_id: u16,
    _version_number: u16,
}

const DUALSENSE_IDS: (u16, u16) = (0x054c, 0x0ce6);
const DUALSENSE_PRODUCT: &str = "DualSense Wireless Controller";

const DUALSENSE_EDGE_IDS: (u16, u16) = (0x054c, 0x0df2);
const DUALSENSE_EDGE_PRODUCT: &str = "DualSense Edge Wireless Controller";

#[apply(proxy!)]
fn HidD_GetAttributes(handle: u32, attrs: *mut HidAttributes) -> bool {
    let ret = unsafe { HidD_GetAttributes_orig(handle, attrs) };
    let attrs = unsafe { &mut *attrs };
    if (attrs.vendor_id, attrs.product_id) == DUALSENSE_EDGE_IDS {
        attrs.vendor_id = DUALSENSE_IDS.0;
        attrs.product_id = DUALSENSE_IDS.1;
    }
    ret
}

#[apply(proxy!)]
fn HidD_GetManufacturerString(handle: u32, buffer: *mut u16, buffer_length: u32) -> bool {
    unsafe { HidD_GetManufacturerString_orig(handle, buffer, buffer_length) }
}

#[apply(proxy!)]
fn HidD_GetProductString(handle: u32, buffer: *mut u16, buffer_length: u32) -> bool {
    let ret = unsafe { HidD_GetProductString_orig(handle, buffer, buffer_length) };
    if !ret {
        return false;
    }
    let str = unsafe { WideCStr::from_ptr_str(buffer) };
    if str.to_string_lossy() == DUALSENSE_EDGE_PRODUCT {
        let new = WideCString::from_str(DUALSENSE_PRODUCT).unwrap();
        unsafe { std::ptr::copy(new.as_ptr(), buffer, new.len() + 1) }
    }
    ret
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(hinstance: HANDLE, reason: u32, _: *mut c_void) -> bool {
    true
}
