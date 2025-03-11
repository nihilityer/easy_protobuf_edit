#![warn(clippy::all, rust_2018_idioms)]


#[macro_use]
extern crate rust_i18n;

mod app;
pub use app::EasyProtobufEditApp;


i18n!("locales", fallback = "zh-CN");