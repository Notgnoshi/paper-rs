use jni::objects::JObject;

/// Marker trait for wrapper types that are layout-compatible with `JObject<'local>` and may
/// therefore be borrowed-cast from a `&JObject<'local>`.
///
/// Several dispatch sites in papermc hold a `&JObject<'local>` (the raw event/sender/etc. handed to a
/// JNI trampoline) and need to hand the user code a `&FooRef<'local>` instead -- a wrapper that
/// exposes typed accessors. The conversion is a pointer reinterpret; for that to be sound,
/// `FooRef<'local>` must be `#[repr(transparent)]` over `JObject<'local>`.
///
/// Historically this invariant lived only in `SAFETY:` comments on the cast sites. So now we have a
/// marker trait to formalize it a bit.
///
/// # Safety
///
/// Use a `#[repr(transparent)]` ZST to wrap the JObject.
pub unsafe trait JObjectRepr<'local>: Sized {
    const _LAYOUT_CHECK: () = assert!(
        std::mem::size_of::<Self>() == std::mem::size_of::<JObject<'local>>(),
        "JObjectRepr implementor is not the size of JObject<'local>; missing #[repr(transparent)]?",
    );

    /// Reinterpret a borrowed `&JObject<'local>` as a borrowed `&Self`.
    fn from_jobject_ref<'a>(obj: &'a JObject<'local>) -> &'a Self {
        // the const _: () = assert!() trick is lazily evaluated for associated trait constants, so
        // you have to refer to the const in an impl to get it to trigger eagerly.
        #[allow(clippy::let_unit_value)]
        let _ = Self::_LAYOUT_CHECK;

        unsafe { &*(obj as *const JObject<'local> as *const Self) }
    }
}
