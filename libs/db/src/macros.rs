/// Returns a human-readable note describing where SQL query wrappers should live.
///
/// This is intentionally a normal function instead of a macro because the scaffold focuses
/// on structure and documentation, not compile-time SQL helper implementation.
pub fn query_wrapper_notes() -> &'static str {
    "Add SQL helper macros or wrapper functions here once the query strategy is finalized."
}
