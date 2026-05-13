use jni::{Env, EnvUnowned};

/// Bridge a C-ABI `extern "C"` function across the JNI boundary into safe Rust.
///
/// Null-checks `env`, attaches a JNI frame, runs `body` inside it, catches Rust panics, and
/// converts any failure (body `Err`, JNI error, panic) into a thrown Java `RuntimeException`
/// carrying the formatted error chain.
///
/// When this function returns `Err`, a Java exception is already pending on the calling thread.
/// The JNI calling convention requires the caller's `extern "C"` function to return an
/// uninterpreted sentinel value (`JNI_FALSE`, `std::ptr::null()`, `0`, etc.) - Java ignores the
/// return value once an exception is pending. The `eyre::Report` carried in `Err` is provided for
/// the caller's reference only; it has already been logged and surfaced to Java.
///
/// A null `env` is treated as an unrecoverable error (no frame to throw on); we log and return
/// `Err` without raising an exception. In practice this can only happen if the JVM passes a null
/// env, which is a JVM bug and not something callers can sensibly handle.
pub(crate) fn bridge<R, F>(env: *mut jni::sys::JNIEnv, body: F) -> eyre::Result<R>
where
    F: FnOnce(&mut Env<'_>) -> eyre::Result<R>,
{
    if env.is_null() {
        let err = eyre::eyre!("ffi entry called with null JNIEnv; cannot raise Java exception");
        tracing::error!("{err:?}");
        return Err(err);
    }

    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<eyre::Result<R>> { Ok(body(env)) })
        .into_outcome();

    let body_result: eyre::Result<R> = match outcome {
        jni::Outcome::Ok(r) => r,
        jni::Outcome::Err(e) => Err(eyre::eyre!("JNI error before body could run: {e}")),
        jni::Outcome::Panic(p) => Err(eyre::eyre!("Rust panic across FFI boundary: {p:?}")),
    };

    match body_result {
        Ok(r) => Ok(r),
        Err(e) => {
            tracing::error!("{e:?}");
            // Surface the error to Java. A second EnvUnowned for the same raw pointer is fine
            // within a single JNI native invocation; we just need the env again to call `throw`.
            let mut unowned = unsafe { EnvUnowned::from_raw(env) };
            let _ = unowned
                .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
                    env.throw(format!("{e:?}"))?;
                    Ok(())
                })
                .into_outcome();
            Err(e)
        }
    }
}
