use jni::objects::JObject;

/// # Safety
///
/// Implementor must be a `#[repr(transparent)]` wrapper over `JObject<'local>`. The trait's default
/// methods rely on this for sound pointer reinterpretation.
///
/// A manual `Drop` impl on the wrapper must not do anything beyond what `JObject<'local>`'s `Drop`
/// already does; otherwise [`JObjectRepr::from_jobject`] will not double-drop but will silently
/// substitute the wrapper's drop for the JObject's.
pub unsafe trait JObjectRepr<'local>: Sized {
    const _LAYOUT_CHECK: () = assert!(
        std::mem::size_of::<Self>() == std::mem::size_of::<JObject<'local>>(),
        "JObjectRepr implementor is not the size of JObject<'local>; missing #[repr(transparent)]?",
    );

    fn from_jobject_ref<'a>(obj: &'a JObject<'local>) -> &'a Self {
        let _ = Self::_LAYOUT_CHECK;
        unsafe { &*(obj as *const JObject<'local> as *const Self) }
    }

    /// # Safety
    ///
    /// `obj` must be a JNI ref to a Java instance of the wrapped class.
    unsafe fn from_jobject(obj: JObject<'local>) -> Self {
        let _ = Self::_LAYOUT_CHECK;
        let obj = std::mem::ManuallyDrop::new(obj);
        unsafe { std::ptr::read(&*obj as *const JObject<'local> as *const Self) }
    }

    fn as_jobject(&self) -> &JObject<'local> {
        let _ = Self::_LAYOUT_CHECK;
        unsafe { &*(self as *const Self as *const JObject<'local>) }
    }
}

/// Identifies a wrapper type as a valid target for `is_instance_of`-based narrowing.
///
/// # Safety
///
/// `CLASS_NAME` must be the JVM descriptor of the exact Java class the wrapper represents (e.g.
/// `"org/bukkit/entity/Player"`).
pub unsafe trait JClassCast<'local>: JObjectRepr<'local> {
    const CLASS_NAME: &'static str;
}
