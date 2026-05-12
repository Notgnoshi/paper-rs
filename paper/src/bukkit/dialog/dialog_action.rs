use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Key;

/// Wrapper for an `io.papermc.paper.registry.data.dialog.action.DialogAction` JNI reference.
///
/// Use the static factory methods to construct concrete actions. The `customClick(BiConsumer,
/// ClickCallback.Options)` variant requires the Rust-closure-from-Java-functional-interface
/// bridge and is deferred to stage 4.
#[repr(transparent)]
pub struct DialogAction<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> DialogAction<'local> {
    /// Construct a custom-click action keyed by `key`, with no NBT payload (null
    /// `BinaryTagHolder`).
    ///
    /// Mirrors `DialogAction.customClick(Key, @Nullable BinaryTagHolder)`.
    pub fn custom_click(api: &mut Api<'_, 'local>, key: &Key<'local>) -> jni::errors::Result<Self> {
        let env = api.jni();
        let null_obj = JObject::null();
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/action/DialogAction"),
                jni_str!("customClick"),
                jni_sig!(
                    "(Lnet/kyori/adventure/key/Key;Lnet/kyori/adventure/nbt/api/BinaryTagHolder;)Lio/papermc/paper/registry/data/dialog/action/DialogAction$CustomClickAction;"
                ),
                &[JValue::Object(&key.obj), JValue::Object(&null_obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }
}
