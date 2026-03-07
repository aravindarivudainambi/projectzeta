/// Calculates the attributed cost for a model interaction.
///
/// The exact formula can evolve with pricing tables, but the function boundary keeps the logic
/// centralized for billing transparency.
pub fn calculate_cost(tokens_in: u64, tokens_out: u64, price_per_token: f64) -> f64 {
    (tokens_in + tokens_out) as f64 * price_per_token
}
