pub mod format;

use crate::lib::ui::format::Format;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use std::borrow::Cow;

#[derive(Clone)]
pub struct UI {
	bar_manager: MultiProgress,
}

impl UI {
	pub fn new(quiet: bool) -> UI {
		UI {
			bar_manager: MultiProgress::with_draw_target(match quiet {
				true => ProgressDrawTarget::hidden(),
				false => ProgressDrawTarget::stderr(),
			}),
		}
	}

	pub fn create_bar(
		&mut self,
		length: u64,
		message: impl Into<Cow<'static, str>>,
	) -> ProgressBar {
		self.bar_manager
			.add(ProgressBar::new(length).with_message(message))
	}

	pub fn create_spinner(
		&self,
		message: impl Into<Cow<'static, str>>,
	) -> ProgressBar {
		self.bar_manager
			.add(ProgressBar::new_spinner().with_message(message))
	}

	pub fn remove_bar(
		&mut self,
		bar: ProgressBar,
	) {
		self.bar_manager.remove(&bar);
	}

	pub fn println<I: AsRef<str>>(
		&self,
		msg: I,
		formatter: impl Format,
	) {
		match self.bar_manager.println(formatter.format(msg)) {
			Ok(_) => (),
			Err(_) => (),
		}
	}
}
