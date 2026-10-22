const ICON_PATH: &str = "assets/trame.ico";

fn main() {
    println!("cargo:rerun-if-changed={ICON_PATH}");
    embed_icon();
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
