use crate::lib::ui::format::Format;
use console::style;


#[derive(Default)]
pub struct Ok {}

impl Format for Ok {
	fn format<I: AsRef<str>>(
		&self,
		input: I,
	) -> String {
		format!("{} {}", style("success:").green().bold().bright(), input.as_ref())
	}
}
