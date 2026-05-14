use jni::objects::JObject;
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

/// Mirror of `org.bukkit.DyeColor`.
///
/// Methods that need to pass a DyeColor across JNI use [`as_java`] to look up the corresponding
/// static field.
#[derive(Copy, Clone, Debug)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl DyeColor {
    fn java_name(self) -> &'static JNIStr {
        match self {
            DyeColor::White => jni_str!("WHITE"),
            DyeColor::Orange => jni_str!("ORANGE"),
            DyeColor::Magenta => jni_str!("MAGENTA"),
            DyeColor::LightBlue => jni_str!("LIGHT_BLUE"),
            DyeColor::Yellow => jni_str!("YELLOW"),
            DyeColor::Lime => jni_str!("LIME"),
            DyeColor::Pink => jni_str!("PINK"),
            DyeColor::Gray => jni_str!("GRAY"),
            DyeColor::LightGray => jni_str!("LIGHT_GRAY"),
            DyeColor::Cyan => jni_str!("CYAN"),
            DyeColor::Purple => jni_str!("PURPLE"),
            DyeColor::Blue => jni_str!("BLUE"),
            DyeColor::Brown => jni_str!("BROWN"),
            DyeColor::Green => jni_str!("GREEN"),
            DyeColor::Red => jni_str!("RED"),
            DyeColor::Black => jni_str!("BLACK"),
        }
    }

    /// Resolve to the `org.bukkit.DyeColor` enum constant in the JVM.
    pub(crate) fn as_java<'local>(
        self,
        env: &mut Env<'local>,
    ) -> jni::errors::Result<JObject<'local>> {
        env.get_static_field(
            jni_str!("org/bukkit/DyeColor"),
            self.java_name(),
            jni_sig!("Lorg/bukkit/DyeColor;"),
        )?
        .l()
    }
}
