//! Framework library for Rust Paper plugins.
use jni_sys::{JNIEnv, jboolean, jlong, jobject, jobjectArray};

mod api;
mod builder;
mod core_init;
mod dispatch;
mod logger;
mod registration;

pub use api::Api;
pub use builder::PluginBuilder;
pub use core_init::core_init;
pub use logger::{install_logger, shutdown_logger};

/// ABI version of the `CoreApi` struct.
///
/// Bump when adding fields. Loaders refuse to load plugins with a mismatched version.
pub const CORE_ABI_VERSION: u32 = 1;

/// The function-pointer table that plugins hand back to `paper-loader` at init time.
///
/// paper-loader's stable JNI symbols forward to these function pointers for all per-call work.
#[repr(C)]
pub struct CoreApi {
    pub abi_version: u32,
    /// Size of the CoreApi struct
    ///
    /// Used to detect ABI mismatches when loading plugins compiled against different versions of
    /// this library.
    pub size: u32,
    /// Per-plugin teardown. Returns 0 on success.
    pub shutdown: unsafe extern "C" fn(*mut JNIEnv) -> i32,
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
}
