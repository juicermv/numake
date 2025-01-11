use crate::lib::ui::format::Format;
use console::style;

#[derive(Default)]
pub struct Info {}
impl Format for Info {
	fn format<I: AsRef<str>>(
		&self,
		input: I,
	) -> String {
		format!("{} {}", style("info:").cyan().bold().bright(), input.as_ref())
	}
}
