use std::array;
use std::collections::HashMap;

use super::types::{cast_order, cast_trade, PyOrder, PyTrade};
use bourse_book::types::{Nanos, OrderCount, OrderId, Price, Side, TraderId, Vol};
use bourse_de::Env as BaseEnv;
use numpy::{PyArray1, ToPyArray};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

/// Discrete event simulation environment
///
/// Simulation environment wrapping an orderbook
/// and functionality to update the state of
/// the simulation. This environment is designed
/// for discrete event simulations, where each
/// step agents submit transactions to the market that
/// are shuffled and executed as a batch at the end
/// of each step. Hence there is no guarantee of
/// the ordering of transactions. Agents do not
/// directly alter the state of the market,
/// rather they do by submitting transactions
/// to be processed.
///
/// Examples
/// --------
///
/// .. testcode:: step_sim_docstring
///
///    import bourse
///
///    seed = 101
///    start_time = 0
///    tick_size = 1
///    step_size = 1000
///
///    env = bourse.core.StepEnv(
///        seed, start_time, tick_size, step_size
///    )
///
///    # Create an order to be placed in the
///    # next update
///    order_id = env.place_order(
///        True, 100, 99, price=50
///    )
///
///    # Update the environment
///    env.step()
///
///    # Get price history data
///    bid_price, ask_prices = env.get_prices()
///
#[pyclass]
pub struct StepEnv {
    env: BaseEnv,
    rng: Xoroshiro128StarStar,
}

#[pymethods]
impl StepEnv {
    #[new]
    #[pyo3(signature = (seed, start_time, tick_size, step_size, trading=true))]
    pub fn new(
        seed: u64,
        start_time: Nanos,
        tick_size: Price,
        step_size: Nanos,
        trading: bool,
    ) -> PyResult<Self> {
        let env = BaseEnv::new(start_time, tick_size, step_size, trading);
        let rng = Xoroshiro128StarStar::seed_from_u64(seed);
        Ok(Self { env, rng })
    }

    /// int: Current simulated time
    #[getter]
    pub fn time(&self) -> Nanos {
        self.env.get_orderbook().get_time()
    }

    /// int: Current total ask side volume
    #[getter]
    pub fn ask_vol(&self) -> Vol {
        self.env.level_2_data().ask_vol
    }

    /// int: Current ask side touch volume
    #[getter]
    pub fn best_ask_vol(&mut self) -> Vol {
        self.env.level_2_data().ask_price_levels[0].0
    }

    /// tuple[int, int]: Current ask touch volume and order count
    #[getter]
    pub fn best_ask_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.env.level_2_data().ask_price_levels[0]
    }

    /// int: Current total bid side volume
    #[getter]
    pub fn bid_vol(&self) -> Vol {
        self.env.level_2_data().bid_vol
    }

    /// int: Current bid side touch volume
    #[getter]
    pub fn best_bid_vol(&mut self) -> Vol {
        self.env.level_2_data().bid_price_levels[0].0
    }

    /// tuple[int, int]: Current bid touch volume and order count
    #[getter]
    pub fn best_bid_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.env.level_2_data().bid_price_levels[0]
    }

    /// int: Trade volume in the last step
    #[getter]
    pub fn trade_vol(&self) -> Vol {
        self.env.get_orderbook().get_trade_vol()
    }

    /// tuple[int, int]: Current bid-ask prices
    #[getter]
    pub fn bid_ask(&mut self) -> (Price, Price) {
        (
            self.env.level_2_data().bid_price,
            self.env.level_2_data().ask_price,
        )
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
        self.env.get_orderbook().order(order_id).status.into()
    }

    /// Enable trading
    ///
    /// When enabled order will be matched and executed.
    ///
    pub fn enable_trading(&mut self) {
        self.env.enable_trading();
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
        self.env.disable_trading();
    }

    /// Update the state of the environment
    ///
    /// Perform one `step` of the simulation updating it's
    /// state, each update performs the following steps
    ///
    /// - Shuffle the order of transactions in the current
    ///   queue
    /// - Execute the instructions in sequence
    /// - Update the market time
    /// - Record the new state of the market
    ///
    /// Transactions should be submitted by agents
    /// prior to calling ``step``, where all
    /// transactions currently in the queue will be
    /// processed.
    ///
    pub fn step(&mut self) -> PyResult<()> {
        self.env.step(&mut self.rng);
        Ok(())
    }

    /// place_order(bid: bool, vol: int, trader_id: int, price: int = None) -> int
    ///
    /// Submit a new-order transaction
    ///
    /// Creates a new order and submit an instruction
    /// to place the order on the market.
    ///
    /// Parameters
    /// ----------
    /// bid: bool
    ///     If ``True`` the order will be placed on the
    ///     bid side.
    /// vol: int
    ///     Volume of the order.
    /// trader_id: int
    ///     Id of the agent/trader placing the order.
    /// price: int, optional
    ///     Limit price of the order, if omitted then
    ///     the order will be treated as a market order.
    ///
    #[pyo3(signature = (bid, vol, trader_id, price=None))]
    pub fn place_order(
        &mut self,
        bid: bool,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> PyResult<OrderId> {
        let side = match bid {
            true => Side::Bid,
            false => Side::Ask,
        };
        let order_id = self.env.place_order(side, vol, trader_id, price);

        match order_id {
            Ok(i) => Ok(i),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    /// cancel_order(order_id: int)
    ///
    /// Submit a cancel order transaction
    ///
    /// Parameters
    /// ----------
    /// order_id: int
    ///     Id of the order to cancel
    ///
    pub fn cancel_order(&mut self, order_id: OrderId) -> PyResult<()> {
        self.env.cancel_order(order_id);
        Ok(())
    }

    /// modify_order(order_id: int, new_price: int = None, new_vol: int = None)
    ///
    /// Submit an transaction to modify an order
    ///
    /// Submit a transaction to modify the price and/or
    /// the volume of an order. Only reducing the volume
    /// of an order us done in place, otherwise the order
    /// will be replaced.
    ///
    /// Parameters
    /// ----------
    /// order_id: int
    ///     Id of the order to modify
    /// new_price: int, optional
    ///     Price to change the order to, if omitted the
    ///     order will remain at its original price.
    /// new_vol: int, optional
    ///     Volume to change the order to, if omitted the
    ///     order will remain at its current volume.
    ///
    #[pyo3(signature = (order_id, new_price=None, new_vol=None))]
    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_price: Option<Price>,
        new_vol: Option<Vol>,
    ) -> PyResult<()> {
        self.env.modify_order(order_id, new_price, new_vol);
        Ok(())
    }

    /// get_prices() -> tuple[numpy.ndarray, numpy.ndarray]
    ///
    /// Get touch price histories
    ///
    /// Returns
    /// -------
    /// tuple
    ///     Tuple containing bid and ask price histories respectively.
    ///
    pub fn get_prices<'a>(&self, py: Python<'a>) -> (&'a PyArray1<Price>, &'a PyArray1<Price>) {
        let prices = self.env.get_prices();
        (prices.0.to_pyarray(py), prices.1.to_pyarray(py))
    }

    /// get_volumes() -> tuple[numpy.ndarray, numpy.ndarray]
    ///
    /// Get volume histories
    ///
    /// Returns
    /// -------
    /// tuple[np.array, np.array]
    ///     Tuple containing histories of bid and ask volumes.
    ///
    pub fn get_volumes<'a>(&self, py: Python<'a>) -> (&'a PyArray1<Vol>, &'a PyArray1<Vol>) {
        let volumes = self.env.get_volumes();
        (volumes.0.to_pyarray(py), volumes.1.to_pyarray(py))
    }

    /// get_touch_volumes() -> tuple[numpy.ndarray, numpy.ndarray]
    ///
    /// Get touch volume histories
    ///
    /// Returns
    /// -------
    /// tuple[np.array, np.array]
    ///     Tuple containing histories of bid and ask touch volumes.
    ///
    pub fn get_touch_volumes<'a>(&self, py: Python<'a>) -> (&'a PyArray1<Vol>, &'a PyArray1<Vol>) {
        let touch_volumes = self.env.get_touch_volumes();
        (
            touch_volumes.0.to_pyarray(py),
            touch_volumes.1.to_pyarray(py),
        )
    }

    /// get_touch_order_counts() -> tuple[numpy.ndarray, numpy.ndarray]
    ///
    /// Get touch volume histories
    ///
    /// Returns
    /// -------
    /// tuple[np.array, np.array]
    ///     Tuple containing histories of bid and ask touch volumes.
    ///
    pub fn get_touch_order_counts<'a>(
        &self,
        py: Python<'a>,
    ) -> (&'a PyArray1<OrderCount>, &'a PyArray1<OrderCount>) {
        let touch_order_counts = self.env.get_touch_order_counts();
        (
            touch_order_counts.0.to_pyarray(py),
            touch_order_counts.1.to_pyarray(py),
        )
    }

    /// level_1_data_array() -> numpy.ndarray
    ///
    /// Get current level 1 data as a Numpy array
    ///
    /// Returns a Numpy array with values at indices:
    ///
    /// - 0: Bid touch price
    /// - 1: Ask touch price
    /// - 2: Bid total volume
    /// - 3: Ask total volume
    /// - 4: Bid touch volume
    /// - 5: Number of buy orders at touch
    /// - 6: Ask touch volume
    /// - 7: Number of sell orders at touch
    ///
    pub fn level_1_data_array<'a>(&self, py: Python<'a>) -> &'a PyArray1<u32> {
        let data = self.env.level_2_data();
        let data_vec = [
            data.bid_price,
            data.ask_price,
            data.ask_vol,
            data.bid_vol,
            data.bid_price_levels[0].0,
            data.bid_price_levels[0].1,
            data.ask_price_levels[0].0,
            data.ask_price_levels[0].1,
        ];

        data_vec.to_pyarray(py)
    }

    /// level_2_data_array() -> numpy.ndarray
    ///
    /// Get current level 2 data as a Numpy array
    ///
    /// Returns a Numpy array with values at indices:
    ///
    /// - 0: Bid touch price
    /// - 1: Ask touch price
    /// - 2: Bid total volume
    /// - 3: Ask total volume
    /// - 4: Bid touch volume
    ///
    /// the following 40 values are data for each
    /// price level below/above the touch
    ///
    /// - Bid volume at level
    /// - Number of buy orders at level
    /// - Ask volume at level
    /// - Number of sell orders at level
    ///
    pub fn level_2_data_array<'a>(&self, py: Python<'a>) -> &'a PyArray1<u32> {
        let data = self.env.level_2_data();
        let mut data_vec = vec![data.bid_price, data.ask_price, data.ask_vol, data.bid_vol];
        for i in 0..10 {
            data_vec.push(data.bid_price_levels[i].0);
            data_vec.push(data.bid_price_levels[i].1);
            data_vec.push(data.ask_price_levels[i].0);
            data_vec.push(data.ask_price_levels[i].1);
        }

        data_vec.to_pyarray(py)
    }

    /// get_trade_volumes() -> numpy.ndarray
    ///
    /// Get trade volume history
    ///
    /// Returns
    /// -------
    /// np.ndarray
    ///     Array tracking the trade volume at each simulation step.
    ///
    pub fn get_trade_volumes<'a>(&self, py: Python<'a>) -> &'a PyArray1<Vol> {
        self.env.get_trade_vols().to_pyarray(py)
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
    ///      with fields:
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
    pub fn get_orders(&self) -> Vec<PyOrder> {
        self.env.get_orders().into_iter().map(cast_order).collect()
    }

    /// get_trades() -> list[tuple]
    ///
    /// Get trade data
    ///
    /// Get a list of trades executed in the environment.
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
    pub fn get_trades(&self) -> Vec<PyTrade> {
        self.env.get_trades().iter().map(cast_trade).collect()
    }

    /// get_market_data() -> dict[str, numpy.ndarray]
    ///
    /// Get simulation market data
    ///
    /// Get a dictionary containing level 2 market data over the simulation
    ///
    /// - Bid and ask touch prices
    /// - Bid and ask volumes
    /// - Volumes and number of orders at 10 levels from the touch
    ///
    /// Returns
    /// -------
    /// dict[str, np.ndarray]
    ///     Dictionary containing level 1 data with keys:
    ///
    ///     - ``bid_price`` - Touch price
    ///     - ``ask_price`` - Touch price
    ///     - ``bid_vol`` - Total volume
    ///     - ``ask_vol`` - Total volume
    ///     - ``trade_vol`` - Total trade vol over a step
    ///     - ``bid_vol_<N>`` - Volumes at 10 levels from bid touch
    ///     - ``ask_vol_<N>`` - Volumes at 10 levels from ask touch
    ///     - ``n_bid_<N>`` - Number of orders at 10 levels from the bid
    ///     - ``n_ask_<N>`` - Number of orders at 10 levels from the ask
    ///
    pub fn get_market_data<'a>(&self, py: Python<'a>) -> HashMap<String, &'a PyArray1<u32>> {
        let data = self.env.get_level_2_data_history();
        let trade_volumes = self.get_trade_volumes(py);

        let bid_vols: [(String, &'a PyArray1<u32>); 10] = array::from_fn(|i| {
            (
                format!("bid_vol_{i}"),
                data.volumes_at_levels.0[i].to_pyarray(py),
            )
        });
        let ask_vols: [(String, &'a PyArray1<u32>); 10] = array::from_fn(|i| {
            (
                format!("ask_vol_{i}"),
                data.volumes_at_levels.1[i].to_pyarray(py),
            )
        });

        let bid_orders: [(String, &'a PyArray1<u32>); 10] = array::from_fn(|i| {
            (
                format!("n_bid_{i}"),
                data.orders_at_levels.0[i].to_pyarray(py),
            )
        });
        let ask_orders: [(String, &'a PyArray1<u32>); 10] = array::from_fn(|i| {
            (
                format!("n_ask_{i}"),
                data.orders_at_levels.1[i].to_pyarray(py),
            )
        });

        let mut py_data = HashMap::from([
            ("bid_price".to_string(), data.prices.0.to_pyarray(py)),
            ("ask_price".to_string(), data.prices.1.to_pyarray(py)),
            ("bid_vol".to_string(), data.volumes.0.to_pyarray(py)),
            ("ask_vol".to_string(), data.volumes.1.to_pyarray(py)),
            ("trade_vol".to_string(), trade_volumes),
        ]);

        py_data.extend(bid_vols);
        py_data.extend(ask_vols);

        py_data.extend(bid_orders);
        py_data.extend(ask_orders);

        py_data
    }
}
