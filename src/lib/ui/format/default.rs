use crate::lib::ui::format::Format;


#[derive(Default)]
pub struct Default {}

impl Format for Default {
	fn format<I: AsRef<str>>(
		&self,
		input: I,
	) -> String {
		input.as_ref().to_string()
	}
}
