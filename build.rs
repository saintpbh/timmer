fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        embed_resource::compile("AppIcon.rc", embed_resource::NONE);
    }
}
