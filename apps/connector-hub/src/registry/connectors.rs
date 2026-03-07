/// Returns the catalog of connectors exposed by the platform.
///
/// The real implementation should likely come from configuration or persistent storage, but
/// a documented function boundary makes the intended responsibility explicit.
pub fn available_connectors() -> Vec<&'static str> {
    vec!["slack", "google_workspace", "salesforce", "notion"]
}
