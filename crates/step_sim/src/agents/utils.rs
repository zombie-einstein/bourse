//! Agent behaviour utilities

use crate::types::Price;

/// Round a price up to the nearest tick and cast to an [Price]
///
/// Can be used to cast samples from continuous distributions
/// to the nearest integer tick for order placement
///
/// # Arguments
///
/// - `p` - Float price
/// - `tick_size` - Tick size as a float
///
pub fn round_price_up(p: f64, tick_size: f64) -> Price {
    let p = (p / tick_size).ceil() * tick_size;
    let p = p.clamp(0.0, Price::MAX.into());
    p as Price
}

/// Round a price down to the nearest tick and cast to an [Price]
///
/// Can be used to cast samples from continuous distributions
/// to the nearest integer tick for order placement
///
/// # Arguments
///
/// - `p` - Float price
/// - `tick_size` - Tick size as a float
///
pub fn round_price_down(p: f64, tick_size: f64) -> Price {
    let p = (p / tick_size).floor() * tick_size;
    let p = p.clamp(0.0, Price::MAX.into());
    p as Price
}

#[cfg(test)]
mod test {
    use super::*;
    use bourse_book::types::Price;

    #[test]
    fn test_rounding_up() {
        let p = round_price_up(5.0, 2.0);
        assert!(p == 6);

        let p = round_price_up(2.1, 2.0);
        assert!(p == 4);

        let p = round_price_up(3.9, 4.0);
        assert!(p == 4);

        // This should never happen but check in case
        let p = round_price_up(-2.2, 4.0);
        assert!(p == 0);

        // This also should never happen but check in case
        let p = round_price_up(1.0f64 + 2.0f64.powi(32), 4.0);
        assert!(p == Price::MAX);
    }
}
