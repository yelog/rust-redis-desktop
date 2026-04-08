fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");

    #[cfg(target_os = "windows")]
    {
        let mut resources = winres::WindowsResource::new();
        resources.set_icon("icons/icon.ico");
        resources
            .compile()
            .expect("failed to compile Windows resources");
    }
}
