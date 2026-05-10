//! `paper` rlib: the framework library for Rust Paper plugins.
//!
//! Stage 1 of the loader-shim migration. Currently this crate hosts the JNI
//! tracing logger bridge. Stage 2 will add `CoreApi`. Stage 4 will expand it
//! with PluginBuilder, Api, dispatch, and the typed Bukkit wrappers.

use std::sync::OnceLock;

use jni::objects::{JClass, JValue};
use jni::refs::Global;
use jni::{Env, JavaVM, jni_sig, jni_str};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

static JVM: OnceLock<JavaVM> = OnceLock::new();
/// `Global` reference to the `PaperFfiLogger` class. Captured at `install_logger`
/// time when the calling thread has plugin-classloader visibility; cached so we
/// never need to call `FindClass` from a tracing event. Tracing events can be
/// emitted from threads that lack plugin-classloader visibility (the JNI default
/// classloader is the system loader, which doesn't see plugin classes).
static DISPATCHER_CLASS: OnceLock<Global<JClass<'static>>> = OnceLock::new();

/// Install a tracing subscriber that routes events to the Java logger via JNI.
///
/// The dispatcher class must expose a static
/// `dispatch(int level, String target, String message)` method.
/// Filtering is controlled by RUST_LOG (default: info).
///
/// Idempotent: subsequent calls are no-ops.
pub fn install_logger(env: &mut Env) -> jni::errors::Result<()> {
    if JVM.get().is_some() {
        return Ok(());
    }
    let vm = env.get_java_vm()?;
    let class = env.find_class(jni_str!("io/paperrs/shim/PaperFfiLogger"))?;
    let class_global = env.new_global_ref(class)?;
    let _ = JVM.set(vm);
    let _ = DISPATCHER_CLASS.set(class_global);
    install_subscriber();
    Ok(())
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
        let Some(jvm) = JVM.get() else { return };
        let Some(class) = DISPATCHER_CLASS.get() else {
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
