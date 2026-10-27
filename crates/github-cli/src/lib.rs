pub fn provider() -> zix_native_provider::Result<zix_native_provider::Provider> {
    zix_native_provider::Provider::load(
        include_bytes!("../api/routes.json"),
        include_bytes!("../api/policy.json"),
        env!("CARGO_PKG_VERSION"),
    )
}

pub fn guided() -> zix_native_provider::Result<(
    zix_native_provider::Manifest,
    zix_native_provider::GuidanceCatalog,
)> {
    zix_native_provider::guidance::from_provider(
        &provider()?,
        include_bytes!("../api/guidance.json"),
    )
}
