use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::DialogAction;
use crate::api::Api;
use crate::bukkit::Component;

/// Wrapper for an `io.papermc.paper.registry.data.dialog.ActionButton` JNI reference.
///
/// Construct with [`ActionButton::create`] (mirrors the Java static factory) or via the
/// Java-side `ActionButton.builder(...)`. The Builder wrapper is deferred until a caller needs
/// finer control than `create` provides.
#[repr(transparent)]
pub struct ActionButton<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> ActionButton<'local> {
    /// Construct an ActionButton from its full set of parameters.
    ///
    /// Mirrors `ActionButton.create(Component label, @Nullable Component tooltip, int width,
    /// @Nullable DialogAction action)`.
    pub fn create(
        api: &mut Api<'_, 'local>,
        label: &Component<'local>,
        tooltip: Option<&Component<'local>>,
        width: i32,
        action: Option<&DialogAction<'local>>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let null = JObject::null();
        let tooltip_obj: &JObject<'_> = tooltip.map(|c| &c.obj).unwrap_or(&null);
        let action_obj: &JObject<'_> = action.map(|a| &a.obj).unwrap_or(&null);
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/ActionButton"),
                jni_str!("create"),
                jni_sig!(
                    "(Lnet/kyori/adventure/text/Component;Lnet/kyori/adventure/text/Component;ILio/papermc/paper/registry/data/dialog/action/DialogAction;)Lio/papermc/paper/registry/data/dialog/ActionButton;"
                ),
                &[
                    JValue::Object(&label.obj),
                    JValue::Object(tooltip_obj),
                    JValue::Int(width),
                    JValue::Object(action_obj),
                ],
            )?
            .l()?;
        Ok(Self { obj })
    }
}
