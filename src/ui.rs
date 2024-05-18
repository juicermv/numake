use std::{
	io::Result,
	time::Duration,
};

use console::{
	Style,
};
use dialoguer::{
	Input,
	MultiSelect,
};
use indicatif::{
	MultiProgress,
	ProgressBar,
	ProgressDrawTarget,
	ProgressStyle,
};

#[derive(Clone)]
pub struct NumakeUI
{
	pub quiet: bool,
	pub progress_manager: MultiProgress,

	style_ok: Style,
	style_err: Style,
	style_warn: Style,
	style_question: Style,
	style_info: Style,
	style_dim: Style,
}

impl NumakeUI
{
	pub fn new(quiet: bool) -> Self
	{
		NumakeUI {
			quiet,

			progress_manager: MultiProgress::with_draw_target(
				if !quiet {
					ProgressDrawTarget::stdout()
				} else {
					ProgressDrawTarget::hidden()
				},
			),

			style_ok: Style::new().green(),
			style_err: Style::new().red(),
			style_warn: Style::new().yellow(),
			style_question: Style::new().cyan(),
			style_info: Style::new().blue(),
			style_dim: Style::new().dim(),
		}
	}

	pub fn format_ok(
		&self,
		val: String,
	) -> String
	{
		format!(
			"{}{}",
			self.style_ok.apply_to("ok"),
			self.style_dim.apply_to(": ".to_string() + &val)
		)
	}

	pub fn print_ok(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			println!("{}", &self.format_ok(out));
		}

		Ok(())
	}

	pub fn format_warn(
		&self,
		val: String,
	) -> String
	{
		self.style_warn
			.apply_to(format!(
				"{}: {}",
				self.style_warn.apply_to("warning"),
				val
			))
			.to_string()
	}

	pub fn print_warn(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			println!("{}", &self.format_warn(out));
		}

		Ok(())
	}

	pub fn format_err(
		&self,
		val: String,
	) -> String
	{
		self.style_err
			.apply_to(format!("{}: {}", self.style_err.apply_to("error"), val))
			.to_string()
	}

	pub fn print_err(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			println!("{}", &self.format_err(out));
		}

		Ok(())
	}

	pub fn format_info(
		&self,
		val: String,
	) -> String
	{
		format!("{}: {}", self.style_info.apply_to("info"), val)
	}

	pub fn print_info(
		&self,
		out: String,
	) -> Result<()>
	{
		if !self.quiet {
			println!("{}", &self.format_info(out))
		}

		Ok(())
	}

	pub fn format_question(
		&self,
		val: String,
	) -> String
	{
		self.style_question
			.apply_to(format!(
				"{}: {}",
				self.style_question.apply_to("question"),
				val
			))
			.to_string()
	}

	pub fn progress(
		&self,
		length: u64,
	) -> ProgressBar
	{
		let bar = ProgressBar::new(length);
		self.progress_manager.add(bar)
	}

	pub fn spinner(
		&self,
		msg: String,
	) -> ProgressBar
	{
		let spinner = ProgressBar::new_spinner().with_message(msg).with_style(
			ProgressStyle::default_spinner()
				.template("{spinner} {wide_msg} {elapsed_precise}")
				.unwrap()
				.tick_strings(&[
					&self.style_question.apply_to("/").to_string(),
					&self.style_question.apply_to("—").to_string(),
					&self.style_question.apply_to("\\").to_string(),
					&self.style_question.apply_to("|").to_string(),
					"-",
				]),
		);
		spinner.enable_steady_tick(Duration::from_millis(115));
		self.progress_manager.add(spinner)
	}

	pub fn input(
		&self,
		prompt: String,
	) -> String
	{
		Input::new().with_prompt(prompt).interact_text().unwrap()
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
			.interact()
			.unwrap()
			.iter()
			.map(|index| items.get(index.clone()).unwrap().clone())
			.collect()
	}
}
