use std::time::Duration;

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;

/// Wrapper for `net.kyori.adventure.text.event.ClickCallback$Options`.
#[repr(transparent)]
pub struct ClickCallbackOptions<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> ClickCallbackOptions<'local> {
    /// `ClickCallback.Options.builder()`.
    pub fn builder(
        api: &mut Api<'_, 'local>,
    ) -> jni::errors::Result<ClickCallbackOptionsBuilder<'local>> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                jni_str!("net/kyori/adventure/text/event/ClickCallback$Options"),
                jni_str!("builder"),
                jni_sig!("()Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"),
                &[],
            )?
            .l()?;
        Ok(ClickCallbackOptionsBuilder { obj })
    }
}

/// Wrapper for `ClickCallback.Options.Builder`.
#[repr(transparent)]
pub struct ClickCallbackOptionsBuilder<'local> {
    obj: JObject<'local>,
}

impl<'local> ClickCallbackOptionsBuilder<'local> {
    /// `Builder.uses(int)`.
    pub fn uses(self, api: &mut Api<'_, 'local>, count: i32) -> jni::errors::Result<Self> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("uses"),
            jni_sig!("(I)Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"),
            &[JValue::Int(count)],
        )?;
        Ok(self)
    }

    /// `Builder.lifetime(TemporalAmount)`. Converts the Rust Duration to a Java `Duration`,
    /// which `implements TemporalAmount`.
    pub fn lifetime(
        self,
        api: &mut Api<'_, 'local>,
        duration: Duration,
    ) -> jni::errors::Result<Self> {
        let env = api.jni();
        let java_duration = env
            .call_static_method(
                jni_str!("java/time/Duration"),
                jni_str!("ofMillis"),
                jni_sig!("(J)Ljava/time/Duration;"),
                &[JValue::Long(duration.as_millis() as i64)],
            )?
            .l()?;
        env.call_method(
            &self.obj,
            jni_str!("lifetime"),
            jni_sig!(
                "(Ljava/time/temporal/TemporalAmount;)Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"
            ),
            &[JValue::Object(&java_duration)],
        )?;
        Ok(self)
    }

    /// `Builder.build()`.
    pub fn build(
        self,
        api: &mut Api<'_, 'local>,
    ) -> jni::errors::Result<ClickCallbackOptions<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("build"),
                jni_sig!("()Lnet/kyori/adventure/text/event/ClickCallback$Options;"),
                &[],
            )?
            .l()?;
        Ok(ClickCallbackOptions { obj })
    }
}
