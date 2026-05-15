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

/// Define a Bukkit event marker plus its `#[repr(transparent)]` JObject wrapper. The marker is
/// the type passed to `SetupApi::register_event::<Marker>(...)`; the wrapper is what handlers
/// receive.
///
/// ```ignore
/// papermc_event! {
///     pub EntityDamageByEntityEvent => EntityDamageByEntityEventRef
///         = "org/bukkit/event/entity/EntityDamageByEntityEvent";
/// }
///
/// impl<'local> EntityDamageByEntityEventRef<'local> {
///     pub fn entity(&self, api: &mut Api<'_, 'local>) -> eyre::Result<EntityInst<'local>> { ... }
/// }
/// ```
#[macro_export]
macro_rules! papermc_event {
    (
        $(#[$marker_attr:meta])*
        $vis:vis $marker:ident => $wrapper:ident = $class:literal ;
    ) => {
        #[allow(unused_imports)]
        use $crate::jobject_repr::JObjectRepr as _;

        $(#[$marker_attr])*
        #[doc = ""]
        #[doc = concat!(
            "Event marker for `", $class, "`. Pass to `SetupApi::register_event`."
        )]
        $vis struct $marker;

        #[doc = concat!("JNI wrapper for the `", $class, "` event.")]
        #[repr(transparent)]
        $vis struct $wrapper<'local> {
            pub(crate) obj: ::jni::objects::JObject<'local>,
        }

        unsafe impl<'local> $crate::jobject_repr::JObjectRepr<'local> for $wrapper<'local> {}

        impl $crate::bukkit::event::Event for $marker {
            type Wrapper<'local> = $wrapper<'local>;
            const CLASS_NAME: &'static str = $class;
        }
    };
}

/// Define a fluent builder wrapper plus its `build()` method. Setters are not templated; write
/// them in a separate `impl` block.
///
/// ```ignore
/// papermc_builder! {
///     pub DialogBaseBuilder<'local> -> DialogBase<'local>
///         builds "()Lio/papermc/paper/registry/data/dialog/DialogBase;";
/// }
///
/// impl<'local> DialogBaseBuilder<'local> {
///     pub fn pause(self, api: &mut Api<'_, 'local>, value: bool) -> eyre::Result<Self> { ... }
///     // ...
/// }
/// ```
///
/// The target type `$out` is constructed via struct literal (`$out { obj }`), so it must have a
/// field named `obj` accessible from the macro call site.
#[macro_export]
macro_rules! papermc_builder {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident <'local> -> $out:ident <'local>
            builds $build_sig:literal ;
    ) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = concat!("Builder for [`", stringify!($out), "`].")]
        #[repr(transparent)]
        $vis struct $name<'local> {
            pub(crate) obj: ::jni::objects::JObject<'local>,
        }

        unsafe impl<'local> $crate::jobject_repr::JObjectRepr<'local> for $name<'local> {}

        impl<'local> $name<'local> {
            #[doc = concat!("Finalize and return a [`", stringify!($out), "`].")]
            pub fn build(
                self,
                api: &mut $crate::Api<'_, 'local>,
            ) -> ::eyre::Result<$out<'local>> {
                let env = api.jni();
                let obj = env
                    .call_method(
                        &self.obj,
                        ::jni::jni_str!("build"),
                        ::jni::jni_sig!($build_sig),
                        &[],
                    )?
                    .l()?;
                Ok($out { obj })
            }
        }
    };
}

/// Define a Rust enum mirror of a Java enum, plus an `as_java` method that resolves to the
/// corresponding static field on the JVM class.
///
/// ```ignore
/// papermc_enum! {
///     pub DyeColor in "org/bukkit/DyeColor" {
///         White => "WHITE",
///         Orange => "ORANGE",
///         // ...
///     }
/// }
/// ```
#[macro_export]
macro_rules! papermc_enum {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident in $class:literal {
            $( $variant:ident => $java_name:literal ),+ $(,)?
        }
    ) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = concat!("Mirror of the Java enum `", $class, "`.")]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        $vis enum $name {
            $($variant,)+
        }

        impl $name {
            #[doc = concat!("Resolve to the matching `", $class, "` enum constant in the JVM.")]
            pub fn as_java<'local>(
                self,
                env: &mut ::jni::Env<'local>,
            ) -> ::jni::errors::Result<::jni::objects::JObject<'local>> {
                let field = match self {
                    $( Self::$variant => $java_name, )+
                };
                $crate::util::get_static_enum_field(env, $class, field)
            }
        }
    };
}
