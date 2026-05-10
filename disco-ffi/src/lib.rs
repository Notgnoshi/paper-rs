use core::ffi::{c_int, c_uchar};

use paper_rs::LoggerFnPtr;
use tracing::info;

#[unsafe(no_mangle)]
pub extern "C" fn disco_init(log_upcall: LoggerFnPtr) {
    paper_rs::install_upcall(log_upcall);
    paper_rs::install_subscriber();
    info!("disco_init called");
}

#[unsafe(no_mangle)]
pub extern "C" fn disco_ping() -> c_int {
    info!("disco_ping called");
    42
}

/// Marshal name bytes -> `disco_core::hello` -> output bytes.
/// Returns the number of bytes written to `out_ptr`.
#[unsafe(no_mangle)]
pub extern "C" fn disco_hello(
    name_ptr: *const c_uchar,
    name_len: c_int,
    out_ptr: *mut c_uchar,
    out_cap: c_int,
) -> c_int {
    let name_bytes = unsafe { std::slice::from_raw_parts(name_ptr, name_len as usize) };
    let name = std::str::from_utf8(name_bytes).unwrap_or("?");
    let reply = disco_core::hello(name);
    let bytes = reply.as_bytes();
    let n = bytes.len().min(out_cap as usize);
    let out = unsafe { std::slice::from_raw_parts_mut(out_ptr, n) };
    out.copy_from_slice(&bytes[..n]);
    n as c_int
}
