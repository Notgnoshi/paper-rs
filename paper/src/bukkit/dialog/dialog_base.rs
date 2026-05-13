use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

use super::{DialogAfterAction, DialogBody};
use crate::api::Api;
use crate::bukkit::Component;

/// Wrapper for an `io.papermc.paper.registry.data.dialog.DialogBase` JNI reference.
#[repr(transparent)]
pub struct DialogBase<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> DialogBase<'local> {
    /// Start a builder for a DialogBase with the given title Component.
    ///
    /// Mirrors `DialogBase.builder(Component title)`.
    pub fn builder(
        api: &mut Api<'_, 'local>,
        title: &Component<'local>,
    ) -> eyre::Result<DialogBaseBuilder<'local>> {
        let env = api.jni();
        let builder_obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/DialogBase"),
                jni_str!("builder"),
                jni_sig!(
                    "(Lnet/kyori/adventure/text/Component;)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"
                ),
                &[JValue::Object(&title.obj)],
            )?
            .l()?;
        Ok(DialogBaseBuilder { obj: builder_obj })
    }
}

/// Wrapper for `io.papermc.paper.registry.data.dialog.DialogBase.Builder`.
///
/// Each chainable setter makes one JNI call; the underlying Java builder returns `this` so we
/// keep the same JObject reference.
#[repr(transparent)]
pub struct DialogBaseBuilder<'local> {
    obj: JObject<'local>,
}

impl<'local> DialogBaseBuilder<'local> {
    /// `Builder.externalTitle(@Nullable Component)`.
    pub fn external_title(
        self,
        api: &mut Api<'_, 'local>,
        title: Option<&Component<'local>>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let null = JObject::null();
        let title_obj: &JObject<'_> = title.map(|c| &c.obj).unwrap_or(&null);
        env.call_method(
            &self.obj,
            jni_str!("externalTitle"),
            jni_sig!(
                "(Lnet/kyori/adventure/text/Component;)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"
            ),
            &[JValue::Object(title_obj)],
        )?;
        Ok(self)
    }

    /// `Builder.canCloseWithEscape(boolean)`.
    pub fn can_close_with_escape(
        self,
        api: &mut Api<'_, 'local>,
        value: bool,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("canCloseWithEscape"),
            jni_sig!("(Z)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"),
            &[JValue::Bool(value)],
        )?;
        Ok(self)
    }

    /// `Builder.pause(boolean)`.
    pub fn pause(self, api: &mut Api<'_, 'local>, value: bool) -> eyre::Result<Self> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("pause"),
            jni_sig!("(Z)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"),
            &[JValue::Bool(value)],
        )?;
        Ok(self)
    }

    /// `Builder.afterAction(DialogAfterAction)`.
    pub fn after_action(
        self,
        api: &mut Api<'_, 'local>,
        action: DialogAfterAction,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let action_obj = action.as_java(env)?;
        env.call_method(
            &self.obj,
            jni_str!("afterAction"),
            jni_sig!(
                "(Lio/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction;)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"
            ),
            &[JValue::Object(&action_obj)],
        )?;
        Ok(self)
    }

    /// `Builder.body(List<? extends DialogBody>)`.
    pub fn body(
        self,
        api: &mut Api<'_, 'local>,
        body: &[DialogBody<'local>],
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let list = dialog_body_list(env, body)?;
        env.call_method(
            &self.obj,
            jni_str!("body"),
            jni_sig!(
                "(Ljava/util/List;)Lio/papermc/paper/registry/data/dialog/DialogBase$Builder;"
            ),
            &[JValue::Object(&list)],
        )?;
        Ok(self)
    }

    /// `Builder.build()` -- finalize and return the DialogBase.
    pub fn build(self, api: &mut Api<'_, 'local>) -> eyre::Result<DialogBase<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("build"),
                jni_sig!("()Lio/papermc/paper/registry/data/dialog/DialogBase;"),
                &[],
            )?
            .l()?;
        Ok(DialogBase { obj })
    }
}

fn dialog_body_list<'local>(
    env: &mut Env<'local>,
    body: &[DialogBody<'local>],
) -> eyre::Result<JObject<'local>> {
    let list = env.new_object(
        jni_str!("java/util/ArrayList"),
        jni_sig!("(I)V"),
        &[JValue::Int(body.len() as i32)],
    )?;
    for b in body {
        env.call_method(
            &list,
            jni_str!("add"),
            jni_sig!("(Ljava/lang/Object;)Z"),
            &[JValue::Object(&b.obj)],
        )?;
    }
    Ok(list)
}
