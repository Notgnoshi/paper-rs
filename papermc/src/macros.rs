/// Define a `#[repr(transparent)]` JObject wrapper, its `JObjectRepr` + `JClassCast` impls, and
/// empty impls for each facet trait listed. Each facet trait must already be in scope at the
/// call site.
///
/// ```ignore
/// use crate::bukkit::{Audience, CommandSender, Entity};
///
/// papermc_jobject! {
///     pub Player<'local> = "org/bukkit/entity/Player": Entity, CommandSender, Audience;
/// }
/// ```
///
/// User-supplied `#[doc = "..."]` / `///` lines on the macro call are placed *before* the
/// auto-generated technical line, so the user's narrative reads as the primary description.
#[macro_export]
macro_rules! papermc_jobject {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident <'local> = $class:literal $(: $($facet:ident),+ $(,)?)? ;
    ) => {
        #[allow(unused_imports)]
        use $crate::jobject_repr::JObjectRepr as _;
        #[allow(unused_imports)]
        use $crate::jobject_repr::JClassCast as _;

        $(#[$attr])*
        #[doc = ""]
        #[doc = concat!("JNI wrapper for `", $class, "`.")]
        #[repr(transparent)]
        $vis struct $name<'local> {
            pub(crate) obj: ::jni::objects::JObject<'local>,
        }

        unsafe impl<'local> $crate::jobject_repr::JObjectRepr<'local> for $name<'local> {}
        unsafe impl<'local> $crate::jobject_repr::JClassCast<'local> for $name<'local> {
            const CLASS_NAME: &'static str = $class;
        }
        $($(
            impl<'local> $facet<'local> for $name<'local> {}
        )+)?
    };
}

/// Define a type-erased handle wrapper plus a `cast<T>` method bounded on the given facet trait.
/// The facet trait must already be in scope at the call site.
///
/// ```ignore
/// use crate::bukkit::Entity;
///
/// papermc_jobject_inst! {
///     pub EntityInst<'local> = "org/bukkit/entity/Entity": Entity;
/// }
/// ```
#[macro_export]
macro_rules! papermc_jobject_inst {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident <'local> = $class:literal : $bound:ident ;
    ) => {
        #[allow(unused_imports)]
        use $crate::jobject_repr::JObjectRepr as _;
        #[allow(unused_imports)]
        use $crate::jobject_repr::JClassCast as _;

        $(#[$attr])*
        #[doc = ""]
        #[doc = concat!(
            "Type-erased JNI wrapper for `", $class, "`. ",
            "Use [`Self::cast`] to narrow to a more specific subtype."
        )]
        #[repr(transparent)]
        $vis struct $name<'local> {
            pub(crate) obj: ::jni::objects::JObject<'local>,
        }

        unsafe impl<'local> $crate::jobject_repr::JObjectRepr<'local> for $name<'local> {}
        unsafe impl<'local> $crate::jobject_repr::JClassCast<'local> for $name<'local> {
            const CLASS_NAME: &'static str = $class;
        }
        impl<'local> $bound<'local> for $name<'local> {}

        impl<'local> $name<'local> {
            #[allow(dead_code)]
            pub(crate) fn new(obj: ::jni::objects::JObject<'local>) -> Self {
                Self { obj }
            }

            #[doc = concat!(
                "Narrow this `", stringify!($name), "` to a specific subtype `T`. ",
                "Returns `None` if the underlying Java object is not a `T`."
            )]
            pub fn cast<T>(self, api: &mut $crate::Api<'_, 'local>) -> Option<T>
            where
                T: $crate::jobject_repr::JClassCast<'local> + $bound<'local>,
            {
                let class = api.class(T::CLASS_NAME).ok()?;
                let env = api.jni();
                if env.is_instance_of(&self.obj, &class).ok()? {
                    Some(unsafe { T::from_jobject(self.obj) })
                } else {
                    None
                }
            }
        }
    };
}
