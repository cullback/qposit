use lobster::{Balance, Price, Timestamp};
use time::{
    macros::{format_description, offset},
    OffsetDateTime,
};

pub mod about_page;
pub mod event;
pub mod home_page;
pub mod login;
pub mod market;
pub mod open_orders;
pub mod order_form;
pub mod orderbook;
pub mod positions;
pub mod profile;
pub mod signup;

/// Formats a price to a string with two decimals.
/// No precision should be lossed.
pub fn format_price_to_string(price: Price) -> String {
    format!("{:.2}¢", price as f32 / 100.0)
}

pub fn format_balance_to_dollars(balance: Balance) -> String {
    format!("${:.2}", balance as f32 / 10000.0)
}

/// Computes the midpoint of two prices, rounding.
const fn midpoint(a: Price, b: Price) -> Price {
    if a > b {
        a.wrapping_add(a.wrapping_sub(b) / 2)
    } else {
        a.wrapping_sub(b.wrapping_sub(a) / 2)
    }
}

pub fn display_price(
    bid: Option<Price>,
    ask: Option<Price>,
    last: Option<Price>,
    outcome: Option<Price>,
) -> String {
    if let Some(outcome) = outcome {
        let price = format_price_to_string(outcome);
        if outcome == 0 {
            return format!("<kbd class=\"pico-background-red-350\">{price}</kbd>");
        }
        return format!("<kbd class=\"pico-background-green-350\">{price}</kbd>");
    }
    let output = match (bid, ask, last) {
        // if two sided quote, use mid price rounded down
        (Some(bid), Some(ask), _) => format_price_to_string(midpoint(bid, ask)),
        (None, None, Some(price)) => format_price_to_string(price),
        _ => "N/A".to_string(),
    };

    format!("<kbd>{}</kbd>", output)
}

/// Pretty prints a timestamp as a string.
/// e.g. November 10, 2020 12:00:00
pub fn format_timestamp_as_string(timestamp: Timestamp) -> String {
    let date = OffsetDateTime::from_unix_timestamp_nanos(i128::from(timestamp * 1000)).unwrap();
    // convert to eastern time
    let date = date.to_offset(offset!(-5));

    let format = format_description!(
        "[weekday], [month repr:long] [day padding:none], [year] at [hour]:[minute]:[second]"
    );
    date.format(&format).unwrap()
}

#[cfg(test)]
mod tests {
    use super::format_price_to_string;
    use super::format_timestamp_as_string;

    #[test]
    fn test_format_price_to_string() {
        assert_eq!(format_price_to_string(0), "0.00¢");
        assert_eq!(format_price_to_string(1), "0.01¢");
        assert_eq!(format_price_to_string(100), "1.00¢");
        assert_eq!(format_price_to_string(101), "1.01¢");
        assert_eq!(format_price_to_string(10000), "100.00¢");
        assert_eq!(format_price_to_string(10001), "100.01¢");
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = 1730829600_000000;
        let formatted = format_timestamp_as_string(timestamp);
        assert_eq!(formatted, "Tuesday, November 5, 2024 at 13:00:00");
    }
}
