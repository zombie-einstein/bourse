use super::types;
use bourse_book::types::{Nanos, OrderCount, OrderId, Price, Side, TraderId, Vol};
use bourse_book::OrderBook as BaseOrderBook;
use pyo3::prelude::*;

/// Rust orderbook interface
///
/// Python interface to a Rust implementation of an
/// orderbook. Allow orders to be placed and modified.
/// The orderbook also stores data on all orders
/// created on the market, allowing the state of orders
/// to be queried from Python.
///
/// Examples
/// --------
///
/// .. testcode:: book_docstring
///
///    import bourse
///
///    book = bourse.core.OrderBook(0, True)
///
///    # Place a new order
///    order_id = book.place_order(
///        True, 100, 0, price=50
///    )
///
///    # Get touch prices
///    bid, ask = book.bid_ask()
///
///    # Get the status of the order
///    status = book.order_status(order_id)
///
#[pyclass]
pub struct OrderBook(BaseOrderBook);

#[pymethods]
impl OrderBook {
    #[new]
    #[pyo3(signature = (start_time, trading=true))]
    pub fn new(start_time: Nanos, trading: bool) -> PyResult<Self> {
        let inner = BaseOrderBook::new(start_time, trading);
        Ok(Self(inner))
    }

    /// set_time(t: int)
    ///
    /// Set the orderbook time
    ///
    /// Set the time of the orderbook, this is then
    /// used to assign time to events on the
    /// orderbook.
    ///
    /// Parameters
    /// ----------
    /// t: int
    ///     Time in nanoseconds (though arbitrary
    ///     units can be used with out effecting
    ///     behaviour).
    ///
    pub fn set_time(&mut self, t: Nanos) {
        self.0.set_time(t);
    }

    /// Enable trading
    ///
    /// When enabled order will be matched and executed.
    ///
    pub fn enable_trading(&mut self) {
        self.0.enable_trading();
    }

    /// Disable trading
    ///
    /// When disabled orders can be placed and modified
    /// but will not be matched.
    ///
    /// Warnings
    /// --------
    /// There is currently no market uncrossing algorithm
    /// implemented.
    ///
    pub fn disable_trading(&mut self) {
        self.0.disable_trading();
    }

    /// ask_vol() -> int
    ///
    /// Get the current total ask side volume
    ///
    /// Returns
    /// -------
    /// int
    ///     Total available volume on ask side
    ///
    pub fn ask_vol(&self) -> Vol {
        self.0.ask_vol()
    }

    /// best_ask_vol() -> int
    ///
    /// Get the current ask side touch volume
    ///
    /// Returns
    /// -------
    /// int
    ///     Touch volume on the ask side
    ///
    pub fn best_ask_vol(&self) -> Vol {
        self.0.ask_best_vol()
    }

    /// best_ask_vol_and_orders() -> (int, int)
    ///
    /// Get the current ask side touch volume and order count
    ///
    /// Returns
    /// -------
    /// tuple[int, int]
    ///     Touch volume and number of orders on the ask side
    ///
    pub fn best_ask_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.0.ask_best_vol_and_orders()
    }

    /// bid_vol() -> int
    ///
    /// Get the current total bid side volume
    ///
    /// Returns
    /// -------
    /// int
    ///     Total available volume on bid side
    ///
    pub fn bid_vol(&self) -> Vol {
        self.0.bid_vol()
    }

    /// best_bid_vol() -> int
    ///
    /// Get the current bid side touch volume
    ///
    /// Returns
    /// -------
    /// int
    ///     Touch volume on the bid side
    ///
    pub fn best_bid_vol(&self) -> Vol {
        self.0.bid_best_vol()
    }

    /// best_bid_vol_and_orders() -> (int, int)
    ///
    /// Get the current bid side touch volume and order count
    ///
    /// Returns
    /// -------
    /// tuple[int, int]
    ///     Touch volume and number of orders on the bid side
    ///
    pub fn best_bid_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.0.bid_best_vol_and_orders()
    }

    /// bid_ask() -> tuple[int, int]
    ///
    /// Get the current bid-ask prices
    ///
    /// Returns
    /// -------
    /// tuple
    ///     Tuple containing the current bid and
    ///     ask prices
    pub fn bid_ask(&self) -> (Price, Price) {
        self.0.bid_ask()
    }

    /// order_status(order_id: int) -> int
    ///
    /// Get the status of an order
    ///
    /// Parameters
    /// ----------
    /// order_id: int
    ///     Id of the order to query.
    ///
    /// Returns
    /// -------
    /// int
    ///     Status of the order as an integer where:
    ///
    ///     - ``0 = New`` Order has not been placed
    ///       on the market
    ///     - ``1 = Active`` Order is on the market
    ///     - ``2 = Filled`` Order has been filled
    ///     - ``3 = Cancelled`` Order has been
    ///       cancelled
    ///     - ``4 = Rejected`` Order has been
    ///       rejected (e.g. a market order in a
    ///       no-trade period)
    ///
    pub fn order_status(&self, order_id: OrderId) -> u8 {
        types::status_to_int(&self.0.order(order_id).status)
    }

    /// place_order(bid: bool, vol: int, trader_id: int, price: int = None) -> int
    ///
    /// Place a new order on the market
    ///
    /// Create and place a new order, will immediately
    /// place the order on the market, matching if
    /// possible.
    ///
    /// Parameters
    /// ----------
    /// bid: bool
    ///     If ``True`` order will be placed on bid-side
    ///     otherwise on the ask-side
    /// vol: int
    ///     Order volume
    /// trader_id: int
    ///     Id of the agent/trader placing the order
    /// price: int, optional
    ///     Order price, if omitted then order will be
    ///     placed as a market order
    ///
    /// Returns
    /// -------
    /// int
    ///     Id of the new order which can then be used
    ///     to modify and query the state of the order.
    ///
    #[pyo3(signature = (bid, vol, trader_id, price=None))]
    pub fn place_order(
        &mut self,
        bid: bool,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> OrderId {
        let side = match bid {
            true => Side::Bid,
            false => Side::Ask,
        };
        let order_id = self.0.create_order(side, vol, trader_id, price);
        self.0.place_order(order_id);
        order_id
    }

    /// cancel_order(order_id: int)
    ///
    /// Cancel order
    ///
    /// Attempt to cancel the order with the given id
    ///
    /// Notes
    /// -----
    /// Cancelling an order that is already
    /// filled/cancelled/rejected will result
    /// in no change to the market.
    ///
    /// Parameters
    /// ----------
    /// order_id: int
    ///     Id of the order to cancel
    ///
    pub fn cancel_order(&mut self, order_id: OrderId) {
        self.0.cancel_order(order_id);
    }

    /// modify_order(order_id: int, new_price: int = None, new_vol: int = None)
    ///
    /// Modify an order
    ///
    /// Modify the price and/or volume of an order.
    /// Only decreasing the volume of an order will maintain
    /// the priority of the order, otherwise the order will
    /// be replaced. Increasing/decreasing the price can
    /// result in the order being executed.
    ///
    /// Notes
    /// -----
    /// The order will keep its existing order-id
    ///
    /// Parameters
    /// ----------
    /// order_id: int
    ///     Id of the order to modify
    /// new_price: int, optional
    ///     New price of the order. If omitted the
    ///     price of the order remains unchanged.
    /// new_vol: int
    ///     New volume of the order. The order will be
    ///     increased/decreased relative to the current
    ///     available order of the volume.
    ///
    #[pyo3(signature = (order_id, new_price=None, new_vol=None))]
    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_price: Option<Price>,
        new_vol: Option<Vol>,
    ) {
        self.0.modify_order(order_id, new_price, new_vol);
    }

    /// get_trades() -> list[tuple]
    ///
    /// Get trade data
    ///
    /// Get a list of trades executed on the market.
    ///
    /// Returns
    /// -------
    /// list
    ///     A list of tuple trade records with fields
    ///
    ///     - Trade time
    ///     - Side flag (``True`` for bid side)
    ///     - Trade price
    ///     - Trade volume
    ///     - Id of the aggressive order
    ///     - Id of the passive order
    ///
    pub fn get_trades(&self) -> Vec<types::PyTrade> {
        self.0.get_trades().iter().map(types::cast_trade).collect()
    }

    /// get_orders() -> list[tuple]
    ///
    /// Get order data
    ///
    /// Return a list of all orders created for the market
    /// including all completed/cancelled/rejected orders.
    ///
    /// Returns
    /// -------
    /// list
    ///     List of tuples records representing all orders created,
    ///     with fields:
    ///
    ///     - side (``True`` indicates bid-side)
    ///     - status of the order
    ///     - arrival time of the order
    ///     - end time of the order
    ///     - Remaining volume of the order
    ///     - Starting volume of the order
    ///     - Price of the order
    ///     - Id of the trader/agent who placed the order
    ///     - Id of the order
    ///
    pub fn get_orders(&self) -> Vec<types::PyOrder> {
        self.0
            .get_orders()
            .into_iter()
            .map(types::cast_order)
            .collect()
    }
}
