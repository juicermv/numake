use std::{
	io::Result,
	time::Duration,
};

use console::{
	Emoji,
	Style,
	Term,
};
use dialoguer::{
	Input,
	MultiSelect,
};
use indicatif::{
	MultiProgress,
	ProgressBar,
	ProgressDrawTarget,
};

#[derive(Clone)]
pub struct NumakeUI
{
	pub quiet: bool,
	pub stdout: Term,
	pub stderr: Term,
	pub progress_manager: MultiProgress,

	style_ok: Style,
	style_err: Style,
	style_warn: Style,
	style_question: Style,
}

impl NumakeUI
{
	pub fn new(quiet: bool) -> Self
	{
		NumakeUI {
			quiet,
			stdout: Term::stdout(),
			stderr: Term::stderr(),

			progress_manager: MultiProgress::with_draw_target(
				if !quiet {
					ProgressDrawTarget::stdout()
				} else {
					ProgressDrawTarget::hidden()
				},
			),

			style_ok: Style::new().green().bold(),
			style_err: Style::new().red().bold(),
			style_warn: Style::new().yellow().bold(),
			style_question: Style::new().cyan().underlined(),
		}
	}

	pub fn format_ok(
		&self,
		val: String,
	) -> String
	{
		self.style_ok
			.apply_to(format!("{} {}", Emoji("✅", "OK!"), val))
			.to_string()
	}

	pub fn print_ok(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			self.stdout.write_line(&self.format_ok(out))
		} else {
			Ok(())
		}
	}

	pub fn format_warn(
		&self,
		val: String,
	) -> String
	{
		self.style_warn
			.apply_to(format!("{} {}", Emoji("⚠️", "WARNING!"), val))
			.to_string()
	}

	pub fn print_warn(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			self.stdout.write_line(&self.format_warn(out))
		} else {
			Ok(())
		}
	}

	pub fn format_err(
		&self,
		val: String,
	) -> String
	{
		self.style_err
			.apply_to(format!("{} {}", Emoji("⛔", "ERROR!"), val))
			.to_string()
	}

	pub fn print_err(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			self.stderr.write_line(&self.format_err(out))
		} else {
			Ok(())
		}
	}

	pub fn format_info(
		&self,
		val: String,
	) -> String
	{
		format!("{} {}", Emoji("ℹ️", "INFO:"), val)
	}

	pub fn print_info(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			self.stderr.write_line(&self.format_info(out))
		} else {
			Ok(())
		}
	}

	pub fn format_question(
		&self,
		val: String,
	) -> String
	{
		self.style_question
			.apply_to(format!("{} {}", Emoji("❔", "?"), val))
			.to_string()
	}

	pub fn progress(
		&self,
		length: u64,
	) -> ProgressBar
	{
		self.progress_manager.add(ProgressBar::new(length))
	}

	pub fn spinner(
		&self,
		msg: String,
	) -> ProgressBar
	{
		let spinner = ProgressBar::new_spinner();
		spinner.set_message(msg);
		spinner.enable_steady_tick(Duration::from_millis(100));
		self.progress_manager.add(spinner)
	}

	pub fn input(
		&self,
		prompt: String,
	) -> String
	{
		Input::new()
			.with_prompt(prompt)
			.interact_text_on(&self.stdout)
			.unwrap()
	}

	pub fn list_select(
		&self,
		prompt: String,
		items: Vec<String>,
	) -> Vec<String>
	{
		MultiSelect::new()
			.with_prompt(prompt)
			.items(items.as_slice())
			.interact_on(&self.stdout)
			.unwrap()
			.iter()
			.map(|index| items.get(*index).unwrap().clone())
			.collect()
	}
}
