fn main() -> std::process::ExitCode {
    zix_native_provider::entry_guided(
        include_bytes!("../api/routes.json"),
        include_bytes!("../api/policy.json"),
        env!("CARGO_PKG_VERSION"),
        include_bytes!("../api/guidance.json"),
    )
}
