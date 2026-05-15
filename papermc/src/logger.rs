//! Bridges Rust `tracing` events into the JVM's Java logger via a static dispatcher class.
//!
//! Each cdylib that links papermc has its own copy of `tracing-core`'s global dispatch (because
//! `tracing-core` is statically linked into every cdylib), so each cdylib installs its own
//! subscriber: papermc-loader from `JNI_OnLoad`, plugin cdylibs from [`crate::init`].
//!
//! `RUST_LOG` is read once at first cdylib install. Server restart required to pick up changes.

use std::sync::{Arc, OnceLock};

use arc_swap::ArcSwapOption;
use jni::objects::{JClass, JValue};
use jni::refs::Global;
use jni::{Env, JavaVM, jni_sig, jni_str};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

static JVM: OnceLock<JavaVM> = OnceLock::new();
static DISPATCHER_CLASS: ArcSwapOption<Global<JClass<'static>>> = ArcSwapOption::const_empty();

/// Idempotent within a cdylib. Called from papermc-loader's `JNI_OnLoad` and from [`crate::init`].
pub fn install_subscriber(jvm: JavaVM) {
    let _ = JVM.set(jvm);
    static SUBSCRIBER_INIT: std::sync::Once = std::sync::Once::new();
    SUBSCRIBER_INIT.call_once(install_layer);
}

/// Refresh on every plugin enable so the cached `Global` doesn't pin a stale ClassLoader.
pub fn bind_dispatcher(env: &mut Env) -> jni::errors::Result<()> {
    let class = env.find_class(jni_str!("io/papermc/RustTracingSubscriber"))?;
    let class_global = env.new_global_ref(class)?;
    DISPATCHER_CLASS.store(Some(Arc::new(class_global)));
    Ok(())
}

/// Events emitted between `unbind_dispatcher` and the next `bind_dispatcher` no-op silently.
pub fn unbind_dispatcher() {
    DISPATCHER_CLASS.store(None);
}

fn install_layer() {
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
        let class_guard = DISPATCHER_CLASS.load();
        let Some(class) = class_guard.as_ref() else {
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

        // Pack fields into the target string so they survive the Java dispatcher's flat prefix.
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
                &**class,
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

/// `record_debug` catches all field types via the default `record_*` impls.
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
