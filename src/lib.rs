//! A Dreamport module: bytes in, bytes out. See README.md for the ABI. The example offers a decoder and no reader,
//! so it imports nothing; `host` is what a reader uses to read its container.

pub mod host;

use std::alloc::{alloc, dealloc, Layout};

const ABI_VERSION: u32 = 4;
const ALIGN: usize = 16;
const CORRUPT: i64 = -1;
const UNSUPPORTED: i64 = -2;
const BAD_ARGUMENTS: i64 = -5;

#[no_mangle]
pub extern "C" fn dp_abi() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn dp_alloc(len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    match Layout::from_size_align(len as usize, ALIGN) {
        Ok(layout) => unsafe { alloc(layout) as usize as u32 },
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn dp_free(ptr: u32, len: u32) {
    if ptr == 0 || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len as usize, ALIGN) {
        unsafe { dealloc(ptr as usize as *mut u8, layout) }
    }
}

#[no_mangle]
pub extern "C" fn dp_decode(name_ptr: u32, name_len: u32, in_ptr: u32, in_len: u32, out_ptr: u32, out_len: u32) -> i64 {
    if (name_ptr == 0 && name_len != 0) || (in_ptr == 0 && in_len != 0) || (out_ptr == 0 && out_len != 0) || out_len > i32::MAX as u32 {
        return BAD_ARGUMENTS;
    }
    let name: &[u8] = if name_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(name_ptr as usize as *const u8, name_len as usize) } };
    let input: &[u8] = if in_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(in_ptr as usize as *const u8, in_len as usize) } };
    let output: &mut [u8] = if out_len == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out_ptr as usize as *mut u8, out_len as usize) } };
    let wrote = decode(name, input, output);
    if wrote < 0 {
        output.fill(0);
    }
    wrote
}

/// The example decoder: an input exactly as long as the output is copied; anything else is refused. `name` is the
/// decoder the host asks for, one of the names in `module.json`.
fn decode(name: &[u8], input: &[u8], output: &mut [u8]) -> i64 {
    if name != b"example" {
        return UNSUPPORTED;
    }
    if input.len() != output.len() {
        return CORRUPT;
    }
    output.copy_from_slice(input);
    output.len() as i64
}

const DESCRIPTION: &[u8] = br#"{"readers":[],"decoders":["example"]}"#;

/// What this module answers, in its own words: the host routes these names to it and keeps no list of its own.
#[no_mangle]
pub extern "C" fn dp_describe() -> i64 {
    DESCRIPTION.len() as i64
}

#[no_mangle]
pub extern "C" fn dp_output() -> u32 {
    DESCRIPTION.as_ptr() as usize as u32
}
