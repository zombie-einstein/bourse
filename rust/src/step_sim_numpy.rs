use std::array;
use std::collections::HashMap;

use super::types::{cast_order, cast_trade, NumpyInstructions, PyOrder, PyTrade};
use bourse_book::types::{Nanos, OrderId, Price, TraderId, Vol};
use bourse_de::{Env as BaseEnv, OrderError};
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
/// This environment returns data and receives
/// instructions via Numpy arrays.
///
/// Examples
/// --------
///
/// .. testcode:: step_sim_numpy_docstring
///
///    import numpy as np
///    import bourse
///
///    seed = 101
///    start_time = 0
///    tick_size = 1
///    step_size = 1000
///
///    env = bourse.core.StepEnvNumpy(
///        seed, start_time, tick_size, step_size
///    )
///
///    # Submit orders via Numpy arrays
///    order_ids = env.submit_limit_orders(
///        (
///            np.array([True, False]),
///            np.array([10, 20], dtype=np.uint32),
///            np.array([101, 202], dtype=np.uint32),
///            np.array([50, 55], dtype=np.uint32),
///        ),
///    )
///
///    # Update the environment
///    env.step()
///
///    # Cancel orders
///    env.submit_cancellations(order_ids)
///
///    # Get level-2 data history
///    level_2_data = env.get_market_data()
///
#[pyclass]
pub struct StepEnvNumpy {
    env: BaseEnv,
    rng: Xoroshiro128StarStar,
}

#[pymethods]
impl StepEnvNumpy {
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

    /// submit_limit_orders(orders: tuple[numpy.ndarray, numpy.ndarray, numpy.ndarray, numpy.ndarray])
    ///
    /// Submit new limit orders from a Numpy array
    ///
    /// Parameters
    /// ----------
    /// orders: tuple[np.array, np.array, np.array, np.array]
    ///     Tuple of numpy arrays containing
    ///
    ///     - Order side as a bool (``True`` if bid-side)
    ///     - Order volumes
    ///     - Trader ids
    ///     - Order prices
    ///
    pub fn submit_limit_orders<'a>(
        &mut self,
        py: Python<'a>,
        orders: (
            &'a PyArray1<bool>,
            &'a PyArray1<Vol>,
            &'a PyArray1<TraderId>,
            &'a PyArray1<Price>,
        ),
    ) -> PyResult<&'a PyArray1<OrderId>> {
        let orders = (
            orders.0.readonly(),
            orders.1.readonly(),
            orders.2.readonly(),
            orders.3.readonly(),
        );

        let sides = orders.0.as_array();
        let volumes = orders.1.as_array();
        let trader_ids = orders.2.as_array();
        let prices = orders.3.as_array();

        let ids: Result<Vec<OrderId>, OrderError> = (0..orders.0.len())
            .map(|i| {
                self.env
                    .place_order(sides[i].into(), volumes[i], trader_ids[i], Some(prices[i]))
            })
            .collect();

        match ids {
            Ok(i) => Ok(i.to_pyarray(py)),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    /// submit_cancellations(order_ids: numpy.ndarray)
    ///
    /// Submit a Numpy array of order ids to cancel
    ///
    /// Parameters
    /// ----------
    /// order_ids: np.array
    ///     Numpy array of order-ids to be cancelled
    ///
    pub fn submit_cancellations(&mut self, order_ids: &'_ PyArray1<OrderId>) -> PyResult<()> {
        let order_ids = order_ids.readonly();
        let order_ids = order_ids.as_array();

        order_ids.for_each(|id| {
            self.env.cancel_order(*id);
        });

        Ok(())
    }

    /// submit_instructions(instructions: tuple[numpy.ndarray, numpy.ndarray, numpy.ndarray, numpy.ndarray, numpy.ndarray, numpy.ndarray])
    ///
    /// Submit market instructions as a tuple of Numpy arrays. This allows
    /// new limit orders and cancellations to be submitted from a tuple
    /// of Numpy arrays. Values that are not used for instructions (e.g.
    /// order-id for a new-order) can be set to a default value that will be ignored.
    ///
    /// Parameters
    /// ----------
    /// instructions: tuple[np.array, np.array, np.array, np.array, np.array, np.array]
    ///     Tuple of numpy arrays containing:
    ///
    ///     - Instruction type, an integer representing
    ///
    ///       - ``0``: No change/null instruction
    ///       - ``1``: New order
    ///       - ``2``: Cancel order
    ///
    ///     - Order sides (as bool, ``True`` for bid side) (used for new orders)
    ///     - Order volumes (used for new orders)
    ///     - Trader ids (used for new orders)
    ///     - Order prices (used for new orders)
    ///     - Order id (used for cancellations)
    ///
    /// Returns
    /// -------
    /// np.ndarray
    ///     Array of ids of newly placed orders. For cancellations
    ///     or null instructions the default value of a max usize
    ///     is returned.
    ///
    #[allow(clippy::type_complexity)]
    pub fn submit_instructions<'a>(
        &mut self,
        py: Python<'a>,
        instructions: NumpyInstructions,
    ) -> PyResult<&'a PyArray1<OrderId>> {
        let instructions = (
            instructions.0.readonly(),
            instructions.1.readonly(),
            instructions.2.readonly(),
            instructions.3.readonly(),
            instructions.4.readonly(),
            instructions.5.readonly(),
        );

        let action = instructions.0.as_array();
        let sides = instructions.1.as_array();
        let volumes = instructions.2.as_array();
        let trader_ids = instructions.3.as_array();
        let prices = instructions.4.as_array();
        let order_ids = instructions.5.as_array();

        let ids: Result<Vec<OrderId>, OrderError> = (0..instructions.0.len())
            .map(|i| match action[i] {
                0 => Ok(OrderId::MAX),
                1 => self.env.place_order(
                    sides[i].into(),
                    volumes[i],
                    trader_ids[i],
                    Some(prices[i]),
                ),
                2 => {
                    self.env.cancel_order(order_ids[i]);
                    Ok(OrderId::MAX)
                }
                _ => Ok(OrderId::MAX),
            })
            .collect();

        match ids {
            Ok(i) => Ok(i.to_pyarray(py)),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    /// level_1_data() -> numpy.ndarray
    ///
    /// Get current level 1 data as a Numpy array
    ///
    /// Returns a Numpy array with values at indices:
    ///
    /// - 0: Trade volume (in the last step)
    /// - 1: Bid touch price
    /// - 2: Ask touch price
    /// - 3: Bid total volume
    /// - 4: Ask total volume
    /// - 5: Bid touch volume
    /// - 6: Number of buy orders at touch
    /// - 7: Ask touch volume
    /// - 8: Number of sell orders at touch
    ///
    pub fn level_1_data<'a>(&self, py: Python<'a>) -> &'a PyArray1<u32> {
        let data = self.env.level_2_data();
        let data_vec = [
            self.env.get_orderbook().get_trade_vol(),
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

    /// level_2_data() -> numpy.ndarray
    ///
    /// Get current level 2 data as a Numpy array
    ///
    /// Returns a Numpy array with values at indices:
    ///
    /// - 0: Trade volume (in the last step)
    /// - 1: Bid touch price
    /// - 2: Ask touch price
    /// - 3: Bid total volume
    /// - 4: Ask total volume
    ///
    /// the following 40 values are data for each
    /// price level below/above the touch
    ///
    /// - Bid volume at level
    /// - Number of buy orders at level
    /// - Ask volume at level
    /// - Number of sell orders at level
    ///
    pub fn level_2_data<'a>(&self, py: Python<'a>) -> &'a PyArray1<u32> {
        let data = self.env.level_2_data();
        let mut data_vec = vec![
            self.env.get_orderbook().get_trade_vol(),
            data.bid_price,
            data.ask_price,
            data.ask_vol,
            data.bid_vol,
        ];
        for i in 0..10 {
            data_vec.push(data.bid_price_levels[i].0);
            data_vec.push(data.bid_price_levels[i].1);
            data_vec.push(data.ask_price_levels[i].0);
            data_vec.push(data.ask_price_levels[i].1);
        }

        data_vec.to_pyarray(py)
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
        let trade_volumes = self.env.get_trade_vols().to_pyarray(py);

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
