use std::ffi::CString;

use jni::Env;
use jni::objects::JObject;
use jni::signature::RuntimeFieldSignature;
use jni::strings::JNIStr;

use crate::ctx;

/// Look up a static enum field on a Java class. The class is resolved through the per-load class
/// cache so repeated lookups skip `FindClass`.
pub fn get_static_enum_field<'local>(
    env: &mut Env<'local>,
    class_name: &'static str,
    field_name: &str,
) -> jni::errors::Result<JObject<'local>> {
    let class = ctx::cached_class(env, class_name)?;

    let field_cstring = CString::new(field_name).expect("enum field name contains no NUL");
    let field_jni =
        JNIStr::from_cstr(&field_cstring).expect("enum field name is valid modified UTF-8");

    let sig_string = format!("L{class_name};");
    let sig: RuntimeFieldSignature = sig_string.parse()?;

    env.get_static_field(&class, field_jni, sig.field_signature())?
        .l()
}
