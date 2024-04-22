fn main() {
	println!(
		"cargo:rustc-env=TARGET={}",
		std::env::var("TARGET").unwrap()
	);

	println!("cargo:rustc-env=X86_64_PC_WINDOWS_GNU_OPENSSL_NO_VENDOR")
}