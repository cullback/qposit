use lobster::Price;

pub mod about_page;
pub mod home_page;
pub mod login;
pub mod market_page;
pub mod open_orders;
pub mod order_form;
pub mod orderbook;
pub mod positions;
pub mod signup;
pub mod profile;

/// Formats a price to a string with two decimals.
/// No precision should be lossed.
pub fn format_price_to_string(price: Price) -> String {
    format!("{:.2}¢", price as f32 / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price_to_string() {
        assert_eq!(format_price_to_string(0), "0.00¢");
        assert_eq!(format_price_to_string(1), "0.01¢");
        assert_eq!(format_price_to_string(100), "1.00¢");
        assert_eq!(format_price_to_string(101), "1.01¢");
        assert_eq!(format_price_to_string(10000), "100.00¢");
        assert_eq!(format_price_to_string(10001), "100.01¢");
    }
}