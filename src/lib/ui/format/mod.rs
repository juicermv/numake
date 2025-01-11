pub mod default;
pub mod error;
pub mod ok;
pub mod info;

pub trait Format {
    fn format<I: AsRef<str>>(&self, input: I) -> String;
}