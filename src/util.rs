pub fn log(what: &str, quiet: bool) {
	if !quiet {
		println!("{}", what);
	}
}