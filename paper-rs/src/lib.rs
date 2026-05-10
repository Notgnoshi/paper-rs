use core::ffi::{c_int, c_uchar};
use std::sync::OnceLock;

use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// C ABI function pointer the Java side passes at init time.
///
/// Receives (level, target_ptr, target_len, message_ptr, message_len).
pub type LoggerFnPtr = unsafe extern "C" fn(c_int, *const c_uchar, c_int, *const c_uchar, c_int);

static LOGGER: OnceLock<LoggerFnPtr> = OnceLock::new();

/// Install the function pointer the tracing layer will dispatch to.
///
/// Idempotent: subsequent calls are no-ops.
pub fn install_upcall(f: LoggerFnPtr) {
    let _ = LOGGER.set(f);
}

/// Install a tracing subscriber that routes events through the upcall.
/// Filtering is controlled by RUST_LOG (default: info).
pub fn install_subscriber() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(UpcallLayer)
        .try_init();
}

struct UpcallLayer;

impl<S: Subscriber> Layer<S> for UpcallLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let Some(f) = LOGGER.get() else { return };
        let level: c_int = match *event.metadata().level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 2,
            tracing::Level::DEBUG => 3,
            tracing::Level::TRACE => 4,
        };
        let target = event.metadata().target();
        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);
        unsafe {
            f(
                level,
                target.as_ptr(),
                target.len() as c_int,
                message.as_ptr(),
                message.len() as c_int,
            );
        }
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
