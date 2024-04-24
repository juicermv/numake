fn main()
{
	println!(
		"cargo:rustc-env=TARGET={}",
		std::env::var("TARGET").unwrap()
	);

	println!("cargo:rustc-env=RUST_BACKTRACE=1");
	println!("cargo:rustc-env=MIRIFLAGS=-Zmiri-disable-isolation");
}
