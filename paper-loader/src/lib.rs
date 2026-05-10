//! `paper-loader` cdylib: the stable JNI surface that Java loads.
//!
//! Stage 1 of the loader-shim migration: this crate exists but is empty. Stage 2
//! will add the Java_io_paperrs_shim_PaperRs_* symbols and the dlopen forwarding
//! to `disco-core.so`.
