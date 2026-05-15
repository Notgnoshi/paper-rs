use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::{DialogBase, DialogType};
use crate::api::Api;

/// Wrapper for an `io.papermc.paper.dialog.Dialog` JNI reference.
///
/// Java's `Dialog.create` takes a `Consumer<RegistryBuilderFactory<...>>` lambda.
#[repr(transparent)]
pub struct Dialog<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Dialog<'local> {
    /// Construct a Dialog from a DialogBase and DialogType.
    ///
    /// Wraps the papermc `Dialogs.create(DialogBase, DialogType)` helper, which itself calls
    /// `Dialog.create(b -> b.empty().base(base).type(type))`.
    pub fn create(
        api: &mut Api<'_, 'local>,
        base: &DialogBase<'local>,
        type_: &DialogType<'local>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/Dialogs"),
                jni_str!("create"),
                jni_sig!(
                    "(Lio/papermc/paper/registry/data/dialog/DialogBase;Lio/papermc/paper/registry/data/dialog/type/DialogType;)Lio/papermc/paper/dialog/Dialog;"
                ),
                &[JValue::Object(&base.obj), JValue::Object(&type_.obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }
}
