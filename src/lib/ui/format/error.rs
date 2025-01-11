use crate::lib::ui::format::Format;
use console::style;


#[derive(Default)]
pub struct Error {}
impl Format for Error {
	fn format<I: AsRef<str>>(
		&self,
		input: I,
	) -> String {
		format!("{} {}", style("error:").red().bold().bright(), input.as_ref())
	}
}
