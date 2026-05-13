use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::ActionButton;
use crate::api::Api;

/// Wrapper for an `io.papermc.paper.registry.data.dialog.type.DialogType` JNI reference.
///
/// Java's `DialogType` static factories return concrete subtypes (`NoticeType`,
/// `ConfirmationType`, `MultiActionType`, etc.); we wrap them all as `DialogType` since the
/// distinction only matters when passing back into Paper, which accepts the parent type.
///
/// Deferred surfaces: `DialogType.dialogList(...)` (needs registry wrapper),
/// `DialogType.serverLinks(...)` (specialized).
#[repr(transparent)]
pub struct DialogType<'local> {
    pub(crate) obj: JObject<'local>,
}

const TYPE_CLASS: &jni::strings::JNIStr =
    jni_str!("io/papermc/paper/registry/data/dialog/type/DialogType");

impl<'local> DialogType<'local> {
    /// `DialogType.notice()` -- a single "OK"-style notice with the default acknowledge button.
    pub fn notice(api: &mut Api<'_, 'local>) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                TYPE_CLASS,
                jni_str!("notice"),
                jni_sig!("()Lio/papermc/paper/registry/data/dialog/type/NoticeType;"),
                &[],
            )?
            .l()?;
        Ok(Self { obj })
    }

    /// `DialogType.notice(ActionButton)` -- a notice with a custom acknowledge button.
    pub fn notice_with(
        api: &mut Api<'_, 'local>,
        action: &ActionButton<'local>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                TYPE_CLASS,
                jni_str!("notice"),
                jni_sig!(
                    "(Lio/papermc/paper/registry/data/dialog/ActionButton;)Lio/papermc/paper/registry/data/dialog/type/NoticeType;"
                ),
                &[JValue::Object(&action.obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }

    /// `DialogType.confirmation(yes, no)` -- two buttons side by side.
    pub fn confirmation(
        api: &mut Api<'_, 'local>,
        yes: &ActionButton<'local>,
        no: &ActionButton<'local>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                TYPE_CLASS,
                jni_str!("confirmation"),
                jni_sig!(
                    "(Lio/papermc/paper/registry/data/dialog/ActionButton;Lio/papermc/paper/registry/data/dialog/ActionButton;)Lio/papermc/paper/registry/data/dialog/type/ConfirmationType;"
                ),
                &[JValue::Object(&yes.obj), JValue::Object(&no.obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }

    /// `DialogType.multiAction(List<ActionButton>)` -- multiple buttons arranged in a grid.
    ///
    /// The 2-arg Java overload returns a `MultiActionType.Builder`; we call `.build()` on it
    /// here to materialize a `MultiActionType` with default exit-action / columns.
    pub fn multi_action(
        api: &mut Api<'_, 'local>,
        actions: &[ActionButton<'local>],
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let list = action_button_list(env, actions)?;
        let builder = env
            .call_static_method(
                TYPE_CLASS,
                jni_str!("multiAction"),
                jni_sig!(
                    "(Ljava/util/List;)Lio/papermc/paper/registry/data/dialog/type/MultiActionType$Builder;"
                ),
                &[JValue::Object(&list)],
            )?
            .l()?;
        let obj = env
            .call_method(
                &builder,
                jni_str!("build"),
                jni_sig!("()Lio/papermc/paper/registry/data/dialog/type/MultiActionType;"),
                &[],
            )?
            .l()?;
        Ok(Self { obj })
    }
}

/// Build a `java.util.ArrayList<ActionButton>` from a Rust slice.
pub(crate) fn action_button_list<'local>(
    env: &mut jni::Env<'local>,
    actions: &[ActionButton<'local>],
) -> eyre::Result<JObject<'local>> {
    let list = env.new_object(
        jni_str!("java/util/ArrayList"),
        jni_sig!("(I)V"),
        &[JValue::Int(actions.len() as i32)],
    )?;
    for action in actions {
        env.call_method(
            &list,
            jni_str!("add"),
            jni_sig!("(Ljava/lang/Object;)Z"),
            &[JValue::Object(&action.obj)],
        )?;
    }
    Ok(list)
}
