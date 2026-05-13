use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Component;

/// Wrapper for an `io.papermc.paper.registry.data.dialog.body.DialogBody` JNI reference.
///
/// Use the static factory methods to construct concrete body types. Currently only
/// [`DialogBody::plain_message`] is wrapped; `DialogBody.item(...)` requires an `ItemStack`
/// wrapper and is deferred.
#[repr(transparent)]
pub struct DialogBody<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> DialogBody<'local> {
    /// Construct a plain-message body from a Component.
    ///
    /// Mirrors `DialogBody.plainMessage(Component)`.
    pub fn plain_message(
        api: &mut Api<'_, 'local>,
        text: &Component<'local>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/body/DialogBody"),
                jni_str!("plainMessage"),
                jni_sig!(
                    "(Lnet/kyori/adventure/text/Component;)Lio/papermc/paper/registry/data/dialog/body/PlainMessageDialogBody;"
                ),
                &[JValue::Object(&text.obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }

    /// Construct a plain-message body from a Component with an explicit pixel width.
    ///
    /// Mirrors `DialogBody.plainMessage(Component, int)`.
    pub fn plain_message_with_width(
        api: &mut Api<'_, 'local>,
        text: &Component<'local>,
        width: i32,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/body/DialogBody"),
                jni_str!("plainMessage"),
                jni_sig!(
                    "(Lnet/kyori/adventure/text/Component;I)Lio/papermc/paper/registry/data/dialog/body/PlainMessageDialogBody;"
                ),
                &[JValue::Object(&text.obj), JValue::Int(width)],
            )?
            .l()?;
        Ok(Self { obj })
    }
}
