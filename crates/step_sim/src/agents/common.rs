//! Common agent behaviours and utilities
//!

use rand::{Rng, RngCore};
use rand_distr::Distribution;

use crate::types::{OrderId, Price, Side, Status, TraderId, Vol};
use crate::{Env, OrderError};

/// Round a price up to the nearest tick and cast to a [Price]
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

/// Round a price down to the nearest tick and cast to a [Price]
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

/// Filter active orders and randomly cancel them
///
/// Filter a vec of [OrderId] for those that are active and
/// then randomly select orders for cancellation. Returns
/// a list of [OrderId] that will remain active.
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `rng` - Random generator
/// - `orders` - Vector of current order ids
/// - `p_cancel` - Probability orders are cancelled
///
pub fn cancel_live_orders<R: RngCore>(
    env: &mut Env,
    rng: &mut R,
    orders: &[OrderId],
    p_cancel: f32,
) -> Vec<OrderId> {
    let live_orders = orders
        .iter()
        .filter(|x| env.order_status(**x) == Status::Active);

    let (live_orders, to_cancel): (Vec<OrderId>, Vec<OrderId>) = live_orders
        .into_iter()
        .partition(|_| rng.gen::<f32>() > p_cancel);

    for order_id in to_cancel.into_iter() {
        env.cancel_order(order_id);
    }

    live_orders
}

/// Place a buy order a random distance below the mid-price
///
/// <div class="warning">
///
/// Prices are rounded *down* to the nearest tick
///
/// </div>
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `rng` -  Random generator
/// - `price_dist` - Price sampling distribution
/// - `mid_price` - Current mid-price
/// - `tick_size` - Tick size (as a float)
/// - `trade_vol` - Size of the trade
/// - `trader_id` - Id of the trader/agent
///
pub fn place_buy_limit_order<R: RngCore, D: Distribution<f64>>(
    env: &mut Env,
    rng: &mut R,
    price_dist: D,
    mid_price: f64,
    tick_size: f64,
    trade_vol: Vol,
    trader_id: TraderId,
) -> Result<OrderId, OrderError> {
    let dist = price_dist.sample(rng).abs();
    let price = mid_price - dist;
    let price = round_price_down(price, tick_size);
    env.place_order(Side::Bid, trade_vol, trader_id, Some(price))
}

/// Place a sell order a random distance above the mid-price
///
/// <div class="warning">
///
/// Prices are rounded *up* to the nearest tick
///
/// </div>
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `rng` -  Random generator
/// - `price_dist` - Price sampling distribution
/// - `mid_price` - Current mid-price
/// - `tick_size` - Tick size (as a float)
/// - `trade_vol` - Size of the trade
/// - `trader_id` - Id of the trader/agent
///
pub fn place_sell_limit_order<R: RngCore, D: Distribution<f64>>(
    env: &mut Env,
    rng: &mut R,
    price_dist: D,
    mid_price: f64,
    tick_size: f64,
    trade_vol: Vol,
    trader_id: TraderId,
) -> Result<OrderId, OrderError> {
    let dist = price_dist.sample(rng).abs();
    let price = mid_price + dist;
    let price = round_price_up(price, tick_size);
    env.place_order(Side::Ask, trade_vol, trader_id, Some(price))
}

#[cfg(test)]
mod test {
    use super::*;
    use bourse_book::types::Price;
    use rand::SeedableRng;
    use rand_distr::Uniform;
    use rand_xoshiro::Xoroshiro128StarStar;

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

    #[test]
    fn test_rounding_down() {
        let p = round_price_down(5.0, 2.0);
        assert!(p == 4);

        let p = round_price_down(2.1, 2.0);
        assert!(p == 2);

        let p = round_price_down(3.9, 4.0);
        assert!(p == 0);

        // This should never happen but check in case
        let p = round_price_down(-2.2, 4.0);
        assert!(p == 0);

        // This also should never happen but check in case
        let p = round_price_down(1.0f64 + 2.0f64.powi(32), 4.0);
        assert!(p == Price::MAX);
    }

    #[test]
    fn test_cancel_orders() {
        let mut env = Env::new(0, 1, 1_000_000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        let ids: Vec<OrderId> = (0..10)
            .into_iter()
            .map(|x| {
                env.place_order(Side::Bid, 100, 0, Some(50)).unwrap();
                x
            })
            .collect();

        env.step(&mut rng);

        let live_ids = cancel_live_orders(&mut env, &mut rng, &ids, 0.0);
        assert!(live_ids.len() == 10);

        let live_ids = cancel_live_orders(&mut env, &mut rng, &ids, 1.0);
        assert!(live_ids.is_empty());

        env.step(&mut rng);
    }

    #[test]
    fn test_placing_orders() {
        let mut env = Env::new(0, 1, 1_000_000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);
        let price_dist = Uniform::<f64>::new(-100.0, 100.0);
        let mid_price: f64 = 200.0;

        let _buy_id =
            place_buy_limit_order(&mut env, &mut rng, price_dist, mid_price, 5.0, 100, 101);

        let _sell_id =
            place_sell_limit_order(&mut env, &mut rng, price_dist, mid_price, 5.0, 100, 101);

        let buy_order = env.get_orders()[0];

        assert!(matches!(buy_order.side, Side::Bid));
        assert!(buy_order.price % 5 == 0);
        assert!(buy_order.price <= 200);

        let sell_order = env.get_orders()[1];

        assert!(matches!(sell_order.side, Side::Ask));
        assert!(sell_order.price % 5 == 0);
        assert!(sell_order.price >= 200);
    }
}
