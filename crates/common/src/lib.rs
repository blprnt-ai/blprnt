#![warn(unused, unused_crate_dependencies)]

pub mod agent;
pub mod apply_patch;
pub mod blprnt;
pub mod blprnt_dispatch;
pub mod blprnt_settings;
pub mod bun_runtime;
pub mod consts;
pub mod credentials;
pub mod errors;
pub mod event_source;
pub mod memory;
pub mod models;
pub mod openrouter;
pub mod paths;
pub mod personality_files;
pub mod personality_service;
pub mod plan_utils;
pub mod provider_dispatch;
pub mod quick_encrypt;
pub mod sandbox_flags;
pub mod session_dispatch;
pub mod shared;
pub mod skills_utils;
pub mod slack;
pub mod tokenizer;
pub mod tools;

pub use ordered_float::OrderedFloat;

#[macro_export]
macro_rules! before_all {
  (init = $path:path) => {
    #[allow(non_snake_case)]
    #[ctor::ctor]
    fn __BEFORE_ALL__() {
      static ONCE: std::sync::Once = std::sync::Once::new();
      ONCE.call_once(|| {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        rt.block_on(async { $path().await });
      });
    }
  };
}

#[macro_export]
macro_rules! after_all {
  (cleanup = $path:path) => {
    #[allow(non_snake_case)]
    #[ctor::ctor]
    fn __AFTER_ALL__() {
      static ONCE: std::sync::Once = std::sync::Once::new();
      ONCE.call_once(|| {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        rt.block_on(async { $path().await });
      });
    }
  };
}
