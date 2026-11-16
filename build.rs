const ICON_PATH: &str = "assets/trame.ico";

fn main() {
    println!("cargo:rerun-if-changed={ICON_PATH}");
    println!("cargo::rustc-check-cfg=cfg(gnome_ui)");
    if gnome_ui() {
        println!("cargo::rustc-cfg=gnome_ui");
    }
    embed_icon();
}

fn gnome_ui() -> bool {
    let linux = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux");
    linux && std::env::var_os("CARGO_FEATURE_GNOME").is_some()
}

#[cfg(windows)]
fn embed_icon() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ICON_PATH);
    resource
        .compile()
        .expect("failed to embed the executable icon");
}

#[cfg(not(windows))]
fn embed_icon() {}
