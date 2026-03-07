/// Scrubs or masks sensitive input before it is sent to an external model provider.
///
/// The real implementation can combine deterministic regex passes with optional classifier-based
/// enrichment, but the function boundary is enough for the initial scaffold.
pub fn scrub_pii(input: &str) -> String {
    input.to_string()
}
