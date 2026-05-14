//! Bridges Rust `tracing` events into the JVM's Java logger via a static dispatcher class.
//!
//! The subscriber lives here in papermc-loader rather than in a core plugin because papermc-loader.s
//! .so is never unloaded; a subscriber whose `JniLayer` code lives in a `dlclose`-able .so would
//! become unmapped on `/reload` and crash on the next tracing event. Keeping it here makes the
//! subscriber a one-time, process-lifetime install.
//!
//! The dispatcher class (`io.papermc.RustTracingSubscriber`) must expose a static
//! `dispatch(int level, String target, String message)` method. Level mapping: 0=ERROR, 1=WARN,
//! 2=INFO, 3=DEBUG, 4=TRACE. Filtering is controlled by `RUST_LOG` (default: `info`).

use std::sync::{Mutex, Once};

use jni::objects::{JClass, JValue};
use jni::refs::Global;
use jni::{Env, JavaVM, jni_sig, jni_str};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// Cached `JavaVM` handle. The JVM is the same instance across `/reload`, so we fetch it once
/// and keep it.
static JVM: Mutex<Option<JavaVM>> = Mutex::new(None);

/// Global ref to the dispatcher class. Refreshed on every `install` so the cached `Global`
/// doesn't pin a stale `ClassLoader` from a previous load.
static DISPATCHER_CLASS: Mutex<Option<Global<JClass<'static>>>> = Mutex::new(None);

/// Tracks the one-shot subscriber installation. `tracing` only honors a single global default,
/// and `try_init` after the first call is a silent no-op; using `Once` makes the contract
/// explicit and avoids rebuilding the filter on every plugin load.
static SUBSCRIBER_INIT: Once = Once::new();

/// Install (or refresh) the JNI logger bridge.
///
/// First call: caches the JVM, acquires a `Global<JClass>` for the dispatcher, and installs the
/// process-global tracing subscriber. Subsequent calls (on each `/reload`) replace the dispatcher
/// `Global` with a fresh one - the previous one's `Drop` releases its pin on the prior
/// `ClassLoader`, so it can be GC'd along with the unloaded plugin.
///
/// Returns `Err` if any JNI lookup fails; the caller may treat this as best-effort and continue
/// (tracing events just no-op until a subsequent install succeeds).
pub(crate) fn install(env: &mut Env) -> jni::errors::Result<()> {
    {
        let mut jvm = JVM.lock().unwrap();
        if jvm.is_none() {
            *jvm = Some(env.get_java_vm()?);
        }
    }
    let class = env.find_class(jni_str!("io/papermc/RustTracingSubscriber"))?;
    let class_global = env.new_global_ref(class)?;
    *DISPATCHER_CLASS.lock().unwrap() = Some(class_global);

    SUBSCRIBER_INIT.call_once(install_subscriber);
    Ok(())
}

/// Drop the cached dispatcher-class `Global` so its `ClassLoader` can be GC'd on `/reload`.
/// The cached `JavaVM` and the installed subscriber stay live for the process lifetime; tracing
/// events between `shutdown` and the next `install` are silently dropped.
pub(crate) fn shutdown() {
    *DISPATCHER_CLASS.lock().unwrap() = None;
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

        let mut message = String::new();
        let mut fields: Vec<(&'static str, String)> = Vec::new();
        let mut visitor = EventVisitor {
            message: &mut message,
            fields: &mut fields,
        };
        event.record(&mut visitor);

        // Embed structured fields into the target string the Java dispatcher already prints in
        // the prefix. `tracing::info!(id = 7, "loaded")` lands in the log as
        // `[INFO: papermc_loader, id=7] loaded` instead of dropping `id` on the floor.
        let target = if fields.is_empty() {
            event.metadata().target().to_string()
        } else {
            use std::fmt::Write;
            let mut t = String::from(event.metadata().target());
            for (k, v) in &fields {
                let _ = write!(t, ", {k}={v}");
            }
            t
        };

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

/// Splits a tracing event's recorded fields into its message text and the rest.
///
/// `record_debug` is the only method we override; the other `record_*` variants on
/// `tracing::field::Visit` default to calling `record_debug`, so this catches all field types
/// (str, i64, u64, bool, etc.) without per-type plumbing.
struct EventVisitor<'a> {
    message: &'a mut String,
    fields: &'a mut Vec<(&'static str, String)>,
}

impl<'a> tracing::field::Visit for EventVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            let mut s = String::new();
            let _ = write!(s, "{value:?}");
            self.fields.push((field.name(), s));
        }
    }
}
