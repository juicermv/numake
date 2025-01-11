fn main()
{
	#[cfg(debug_assertions)]
	println!("cargo:rustc-env=RUST_BACKTRACE=full");
}
