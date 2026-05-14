This project is a proof-of-concept Rust implementation of a Minecraft Paper server plugin built
through Java's JNI native interface. The goal is to write meaningful Rust Paper plugins with
_minimal_ Java.

# Project layout

* Makefile - main entrypoint for build. It handles the orchestration of Gradle (java) and Cargo
  (rust). Neither Gradle nor Cargo know about each other.

* paper-loader - Provides libpaper_loader.so, which is what the DiscoPlugin loads via paper-shim's
  NativeLoader. paper-loader.so loads disco-core.so, which is where the implementation details of
  the Rust side of the Disco plugin are. We do this so that the /reload command can work, since it's
  not possible to reload a native DSO in Java?

  Ideally, the paper-loader _never_ has to change when we add new functionality to the Disco plugin.
  It's intended to be stable, so that we can run the server once, and /reload once we make
  modification.

* disco-core - the Rust implementation of the Disco plugin's business logic. Provided through
  libdisco_core.so

  Ideally, new features are added here, provided sufficient APIs are provided by the paper Rust
  crate.

* paper - the Rust / Java interface. This is where the JNI interfaces are wrapped. It's shareable
  between plugins.

  Eventually, this will grow to contain a Rust wrapper around the bukkit / paper Java plugin API. As
  the bukkit / paper API surface is very large, it's extremely likely that new features will require
  modifications to the paper crate.

  Ideally, the API provided by the paper Rust crate mirrors the bukkit / paper plugin API so that
  it's fairly natural to write Rust plugins. We'll have to consider modifications when we come up
  against language limitations, but we should strive to mirror the Java APIs as much as possible.

* paper-shim - provides Java utilities for building a Paper plugin in Rust. Provides logging and
  native plugin loading.

  It is not expected to require changes to paper-shim when new features are added.

* disco-plugin - this is the Java side of the disco-core plugin. It uses the paper-shim java library
  to load the Rust implementation of the disco-core plugin
