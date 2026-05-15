//! Framework library for Rust Paper plugins.
use jni_sys::{JNIEnv, jboolean, jlong, jobject, jobjectArray};

mod api;
pub mod bukkit;
pub(crate) mod callbacks;
pub(crate) mod ctx;
mod dispatch;
pub(crate) mod ffi;
pub mod jobject_repr;
pub mod logger;
mod macros;
mod plugin;
mod plugin_init;
mod registration;
mod setup_api;
pub mod util;

pub use api::Api;
pub use plugin::Plugin;
pub use plugin_init::init;
pub use setup_api::SetupApi;

/// ABI version of the `FnTable` struct.
///
/// Bump when adding fields. Loaders refuse to load plugins with a mismatched version.
pub const PLUGIN_ABI_VERSION: u32 = 2;

/// The function-pointer table that plugins hand back to `papermc-loader` at init time.
///
/// papermc-loader's stable JNI symbols forward to these function pointers for all per-call work.
#[repr(C)]
pub struct FnTable {
    pub abi_version: u32,
    /// Size of the FnTable struct
    ///
    /// Used to detect ABI mismatches when loading plugins compiled against different versions of
    /// this library.
    pub size: u32,
    /// Per-plugin teardown; invoked by papermc-loader at `Java_..._on_disable` time. Returns 0
    /// on success.
    pub on_disable: unsafe extern "C" fn(*mut JNIEnv) -> i32,
    /// Bukkit fired an event registered through this core; look up handler by id and invoke it.
    pub dispatch_event: unsafe extern "C" fn(*mut JNIEnv, jlong, jobject),
    /// Bukkit dispatched a command registered through this core.
    ///
    /// Returns JNI_TRUE if handled, JNI_FALSE if Bukkit should print usage.
    pub dispatch_command:
        unsafe extern "C" fn(*mut JNIEnv, jlong, jobject, jobjectArray) -> jboolean,
    /// Tab-completion. Returns a Java `List<String>` or null.
    pub dispatch_tab_complete:
        unsafe extern "C" fn(*mut JNIEnv, jlong, jobject, jobjectArray) -> jobject,
    /// A Java functional-interface bridge (currently DialogActionCallback) was invoked; look up
    /// the Rust closure by id and run it with the two object arguments.
    pub dispatch_bi_consumer: unsafe extern "C" fn(*mut JNIEnv, jlong, jobject, jobject),
    /// Java's Cleaner signalled that a bridge instance was GC'd; drop the Rust closure with the
    /// given id from the callback registry.
    pub drop_callback: unsafe extern "C" fn(jlong),
}
