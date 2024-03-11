use clap::Parser;
use core::ffi::{c_char, c_int};
use std::sync::OnceLock;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

static ARGS: OnceLock<Args> = OnceLock::new();

#[derive(Parser, Debug, Copy, Clone)]
#[command(author, about, long_about = None)]
#[repr(C)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
/// List of possible command line arguments
pub struct Args {
    /// Start the engine in dedicated server mode
    #[arg(short, long, default_value_t = false)]
    pub server: bool,

    /// Display version and copyright info
    #[arg(short, long, default_value_t = false)]
    pub version: bool,
}

/// Parse arguments from C and send to the Clap library
///
/// # Safety
///
/// This only checks if argv is null,
/// it does not verify that argv points to valid data
pub unsafe fn parse_args_from_c(
    argc: c_int,
    argv_pointer: *const *const *const c_char,
) -> Option<Vec<String>> {
    use core::ffi::CStr;

    // Check if argv_pointer is null
    if argv_pointer.is_null() {
        return None;
    }

    // Cast back to *const *const c_char so we can operate on it
    //  now that we passed the Safe API Boundary/Barrier
    let argv: *const *const c_char = argv_pointer as *const *const c_char;

    // Check if argv is null
    if argv.is_null() {
        return None;
    }

    // Parse array out of argv
    let c_args: &[*const c_char] = unsafe { std::slice::from_raw_parts(argv, argc as usize) };

    let mut args: Vec<String> = vec![];
    for &arg in c_args {
        let c_str: &CStr = unsafe { CStr::from_ptr(arg) };
        let str_slice: &str = c_str.to_str().unwrap();

        args.push(str_slice.to_owned());
    }

    Some(args)
}

/// Set parsed args passed in from function
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub fn set_parsed_args(args: Vec<String>) {
    // If we already set the args, don't save again
    // It's a OnceLock, we can only set it once anyway
    if ARGS.get().is_some() {
        return;
    }

    let _ = ARGS.set(Args::parse_from(args.iter()));
}

/// Retrieve parsed args previously passed in from function
pub fn get_args() -> Option<Args> {
    ARGS.get().copied()
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    #[test]
    fn test() -> () {
        use super::*;

        // Null pointer should be disregarded (e.g. return is None)
        unsafe {
            assert_eq!(parse_args_from_c(999, core::ptr::null()), None);
        }

        // Valid argument passed in (e.g. return is vec!["hello"])
        unsafe {
            // Create C String
            let arg_one: CString = CString::new("hello").unwrap();
            let arg_one_ptr: *const c_char = arg_one.as_ptr();

            // Add C String to array
            let argv = [arg_one_ptr];

            // Test Parser
            assert_eq!(
                parse_args_from_c(
                    argv.len() as i32,
                    argv.as_ptr() as *const *const *const c_char
                ),
                Some(vec!["hello".to_owned()])
            );
        }
    }
}
