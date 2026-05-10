//! `paper` rlib: the framework library for Rust Paper plugins.
//!
//! Hosts the JNI tracing logger bridge and the `CoreApi` ABI struct that
//! `paper-loader` and `disco-core` use to communicate across the dlopen boundary.

use std::sync::Mutex;

use jni_sys::{JNIEnv, jboolean, jlong, jobject, jobjectArray};

/// ABI version of the `CoreApi` struct. Bump when adding fields. Loaders refuse
/// to load cores with a mismatched version.
pub const CORE_ABI_VERSION: u32 = 1;

/// The function-pointer table that `disco-core` (and any future plugin core)
/// hands back to `paper-loader` at init time. paper-loader's JNI symbols
/// forward to these function pointers for all per-call work.
///
/// `paper_core_init` (a free `extern "C"` function in the core cdylib) returns
/// `*const CoreApi`. The loader then calls the function pointers in this struct
/// for lifecycle and dispatch.
#[repr(C)]
pub struct CoreApi {
    pub abi_version: u32,
    pub size: u32,
    /// Per-plugin init: install the tracing logger, register event/command
    /// handlers via Bukkit. Returns 0 on success, non-zero on failure.
    pub init: unsafe extern "C" fn(*mut JNIEnv, jobject) -> i32,
    /// Per-plugin teardown. Returns 0 on success.
    pub shutdown: unsafe extern "C" fn(*mut JNIEnv) -> i32,
    /// Bukkit fired an event registered through this core; look up handler by
    /// id and invoke it.
    pub dispatch_event: unsafe extern "C" fn(*mut JNIEnv, jlong, jobject),
    /// Bukkit dispatched a command registered through this core. Returns
    /// JNI_TRUE if handled, JNI_FALSE if Bukkit should print usage.
    pub dispatch_command:
        unsafe extern "C" fn(*mut JNIEnv, jlong, jobject, jobjectArray) -> jboolean,
    /// Tab-completion. Returns a Java `List<String>` (jobject) or null.
    pub dispatch_tab_complete:
        unsafe extern "C" fn(*mut JNIEnv, jlong, jobject, jobjectArray) -> jobject,
}

use jni::objects::{JClass, JValue};
use jni::refs::Global;
use jni::{Env, JavaVM, jni_sig, jni_str};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

static JVM: Mutex<Option<JavaVM>> = Mutex::new(None);
/// `Global` reference to the `PaperFfiLogger` class. Captured at `install_logger`
/// time when the calling thread has plugin-classloader visibility; cached so we
/// never need to call `FindClass` from a tracing event. Tracing events can be
/// emitted from threads that lack plugin-classloader visibility (the JNI default
/// classloader is the system loader, which doesn't see plugin classes).
static DISPATCHER_CLASS: Mutex<Option<Global<JClass<'static>>>> = Mutex::new(None);

/// Install a tracing subscriber that routes events to the Java logger via JNI.
///
/// The dispatcher class must expose a static
/// `dispatch(int level, String target, String message)` method.
/// Filtering is controlled by RUST_LOG (default: info).
///
/// Re-entrant: if called again with logger state already set, no-ops.
pub fn install_logger(env: &mut Env) -> jni::errors::Result<()> {
    {
        let jvm_lock = JVM.lock().unwrap();
        if jvm_lock.is_some() {
            return Ok(());
        }
    }
    let vm = env.get_java_vm()?;
    let class = env.find_class(jni_str!("io/paperrs/shim/PaperFfiLogger"))?;
    let class_global = env.new_global_ref(class)?;
    *JVM.lock().unwrap() = Some(vm);
    *DISPATCHER_CLASS.lock().unwrap() = Some(class_global);
    install_subscriber();
    Ok(())
}

/// Tear down the JNI logger bridge: drops the cached `Global<JClass>` reference
/// (which calls `DeleteGlobalRef`, releasing the JVM-side pin on the class) and
/// clears the cached `JavaVM` handle. Subsequent tracing events become no-ops
/// until `install_logger` is called again on the next plugin enable.
///
/// Must be called BEFORE the core .so is dlclose'd; after dlclose the `Global`
/// drop code (in core's `paper` rlib copy) would be unmapped.
pub fn shutdown_logger() {
    *DISPATCHER_CLASS.lock().unwrap() = None;
    *JVM.lock().unwrap() = None;
}

fn install_subscriber() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(JniLayer)
        .try_init();
}

struct JniLayer;

impl<S: Subscriber> Layer<S> for JniLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let jvm_lock = JVM.lock().unwrap();
        let Some(jvm) = jvm_lock.as_ref() else { return };
        let class_lock = DISPATCHER_CLASS.lock().unwrap();
        let Some(class) = class_lock.as_ref() else {
            return;
        };

        let level: i32 = match *event.metadata().level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 2,
            tracing::Level::DEBUG => 3,
            tracing::Level::TRACE => 4,
        };

        let target = event.metadata().target().to_string();
        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        let _ = jvm.attach_current_thread(|env: &mut Env| -> jni::errors::Result<()> {
            let target_jstr = env.new_string(&target)?;
            let message_jstr = env.new_string(&message)?;
            env.call_static_method(
                class,
                jni_str!("dispatch"),
                jni_sig!("(ILjava/lang/String;Ljava/lang/String;)V"),
                &[
                    JValue::Int(level),
                    JValue::Object(&target_jstr),
                    JValue::Object(&message_jstr),
                ],
            )?;
            Ok(())
        });
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        if field.name() == "message" {
            let _ = write!(self.0, "{value:?}");
        }
    }
}
