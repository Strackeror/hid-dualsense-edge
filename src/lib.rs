#![feature(naked_functions)]

use std::arch::asm;
use std::os::raw::c_void;
use std::os::windows::raw::HANDLE;

use dll::load_functions;
use macro_rules_attribute::apply;
use widestring::{WideCStr, WideCString};

macro_rules! dll_proxy {
    ($path:literal $($func:ident)*)=> {
        use std::ffi::c_void;
        use std::ptr::null;
        use anyhow::Result;
        use winsafe::prelude::*;
        use winsafe::HINSTANCE;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        pub enum Functions {
            $($func),*
        }

        const LEN: usize = [$(Functions::$func),*].len();

        #[no_mangle]
        pub static mut ORIG_FUNCTIONS: [*const c_void; LEN] = [null(); LEN];

        static mut _PROXY_FUNCTIONS: [*const c_void; LEN] = [ $(&crate::$func as *const _ as _),* ];

        impl Functions {
            pub const fn index(self) -> usize {
                const FUNCTIONS: [Functions; LEN] = [$(Functions::$func),*];

                let mut i = 0;
                while i < LEN {
                    match (self, FUNCTIONS[i]) {
                        $((Functions::$func, Functions::$func) => return i,)*
                        _ => ()
                    }
                    i += 1;
                };
                0
            }

            pub fn get_orig(self) -> *const c_void {
                unsafe { ORIG_FUNCTIONS[self.index()] }
            }
        }

        pub unsafe fn load_functions() -> Result<()> {
            let func_list = unsafe { &mut *std::ptr::addr_of_mut!(ORIG_FUNCTIONS) };

            let mut lib = HINSTANCE::LoadLibrary($path)?;
            $(
                func_list[Functions::$func.index()] = lib.GetProcAddress(stringify!($func))?;
            )*
            _ = lib.leak();
            Ok(())
        }
    };
}

macro_rules! forward {
    ($func:ident) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $func() {
            static mut FUNC: *const *const c_void = unsafe {&dll::ORIG_FUNCTIONS[dll::Functions::$func.index()]} as *const _;
            asm!(
                "mov rax, [rip + {}]",
                "jmp [rax]",
                sym FUNC,
                options(noreturn),
            )
        }
    };
}

macro_rules! proxy {
    (fn $func:ident ($($name:ident : $arg:ty),*) -> $ret:ty {$($t:tt)*}) => {
        #[allow(non_snake_case)]

        #[no_mangle]
        extern "C" fn $func($($name : $arg),*) -> $ret {
            type Call = extern "C" fn($($name : $arg),*) -> $ret;

            pub fn orig() -> Call {
                unsafe { std::mem::transmute(dll::Functions::$func.get_orig()) }
            }
            $($t)*
        }
    };
}

mod dll;

forward!(HidD_FreePreparsedData);
forward!(HidD_SetConfiguration);
forward!(HidP_GetSpecificButtonCaps);
forward!(HidP_GetSpecificValueCaps);
forward!(HidP_GetExtendedAttributes);
forward!(HidP_GetUsages);
forward!(HidP_SetData);
forward!(HidP_MaxUsageListLength);
forward!(HidD_GetNumInputBuffers);
forward!(HidD_FlushQueue);
forward!(HidD_GetConfiguration);
forward!(HidD_GetMsGenreDescriptor);
forward!(HidD_GetPreparsedData);
forward!(HidD_SetOutputReport);
forward!(HidP_GetCaps);
forward!(HidP_GetButtonArray);
forward!(HidP_GetData);
forward!(HidP_GetLinkCollectionNodes);
forward!(HidP_MaxDataListLength);
forward!(HidD_SetFeature);
forward!(HidD_SetNumInputBuffers);
forward!(HidP_GetUsageValue);
forward!(HidP_GetUsagesEx);
forward!(HidD_Hello);
forward!(HidP_GetScaledUsageValue);
forward!(HidD_GetInputReport);
forward!(HidD_GetFeature);
forward!(HidP_GetVersionInternal);
forward!(HidP_InitializeReportForID);
forward!(HidP_SetScaledUsageValue);
forward!(HidP_SetUsageValueArray);
forward!(HidP_SetUsageValue);
forward!(HidP_SetUsages);
forward!(HidP_UsageListDifference);
forward!(HidD_GetPhysicalDescriptor);
forward!(HidP_GetButtonCaps);
forward!(HidD_GetHidGuid);
forward!(HidP_TranslateUsagesToI8042ScanCodes);
forward!(HidP_UnsetUsages);
forward!(HidP_SetButtonArray);
forward!(HidP_GetUsageValueArray);
forward!(HidD_GetIndexedString);
forward!(HidP_GetValueCaps);
forward!(HidD_GetSerialNumberString);

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
    let ret = orig()(handle, attrs);
    let attrs = unsafe { &mut *attrs };
    if (attrs.vendor_id, attrs.product_id) == DUALSENSE_EDGE_IDS {
        attrs.vendor_id = DUALSENSE_IDS.0;
        attrs.product_id = DUALSENSE_IDS.1;
    }
    ret
}

#[apply(proxy!)]
fn HidD_GetManufacturerString(handle: u32, buffer: *mut u16, buffer_length: u32) -> bool {
    orig()(handle, buffer, buffer_length)
}

#[apply(proxy!)]
fn HidD_GetProductString(handle: u32, buffer: *mut u16, buffer_length: u32) -> bool {
    let ret = orig()(handle, buffer, buffer_length);
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
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        return unsafe { load_functions() }.is_ok();
    }
    true
}
