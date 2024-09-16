#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;
pub(crate) mod cmd;
pub(crate) mod filepicker;
pub(crate) mod groups;
