use std::sync::Mutex;

use jni::objects::{JClass, JValue};
use jni::refs::Global;
use jni::{Env, JavaVM, jni_sig, jni_str};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

static JVM: Mutex<Option<JavaVM>> = Mutex::new(None);
static DISPATCHER_CLASS: Mutex<Option<Global<JClass<'static>>>> = Mutex::new(None);

/// Install a tracing subscriber that routes events to the Java logger via JNI.
///
/// The dispatcher class must expose a static `dispatch(int level, String target, String message)`
/// method. Filtering is controlled by RUST_LOG (default: info).
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

/// Tear down the JNI logger bridge
///
/// Drops the cached `Global<JClass>` reference (which calls `DeleteGlobalRef`, releasing the
/// JVM-side pin on the class) and clears the cached `JavaVM` handle. Subsequent tracing events
/// become no-ops until `install_logger` is called again on the next plugin enable.
///
/// Must be called BEFORE the core .so is dlclose'd; after dlclose the `Global` drop code (in core's
/// `paper` rlib copy) would be unmapped.
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
