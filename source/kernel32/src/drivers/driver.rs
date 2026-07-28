// drivers/driver.rs
use core::ffi::{c_char, c_int, c_void};
#[repr(C)]
pub struct KernelIOInterfaces {
    pub print_string: Option<extern "C" fn(str: *const c_char)>,
    pub clear_screen: Option<extern "C" fn()>,
    pub get_char: Option<extern "C" fn() -> u8>,
    pub get_line: Option<extern "C" fn() -> [u8; crate::drivers::keyboard::INPUT_MAX]>,
    pub kmalloc: Option<extern "C" fn(size: u32) -> *mut c_void>,
    pub kfree: Option<extern "C" fn(ptr: *mut c_void)>,
    pub memset: Option<unsafe extern "C" fn(s: *mut c_void, c: c_int, n: usize) -> *mut c_void>,
    pub memcmp: Option<unsafe extern "C" fn(s1: *const c_void, s2: *const c_void, n: usize) -> c_int>,
    pub register_action_symbol: Option<extern "C" fn(name: *const u8, addr: *mut u8) -> i32>,
    pub resolve_symbol: Option<extern "C" fn(name: *const u8) -> *mut u8>,
}

// Заголовок RCOM-файла
#[repr(C, packed)]
pub struct RcomHeader {
    pub magic: [u8; 4],
    pub magic_check: u32,
    pub exec_offset: u32,
    pub component_size: u32,
    pub name: [u8; 16],
}

const RCOM_MAGIC_LE: u32 = 0x52434F4D;   // 'RCOM'
const DRIVER_MAGIC_LE: u32 = 0x53484954; // 'SHIT'

#[macro_export]
macro_rules! realix_component {
    ($comp_name:expr, $init_func:ident) => {
        #[link_section = ".header"]
        #[no_mangle]
        pub static MY_DRIVER_HEADER: $crate::RcomHeader = $crate::RcomHeader {
            magic: *b"RCOM",
            magic_check: DRIVER_MAGIC_LE,
            exec_offset: 32,        
            component_size: 0,   
            name: {
                let mut bytes = [0u8; 16];
                let src = $comp_name.as_bytes();
                let mut i = 0;
                while i < src.len() && i < 15 {
                    bytes[i] = src[i];
                    i += 1;
                }
                bytes
            },
        };

        #[link_section = ".text.entry"]
        #[no_mangle]
        pub unsafe extern "C" fn driver_init(io: &$crate::KernelIOInterfaces) -> i32 {
            $init_func(io)
        }
    };
}
