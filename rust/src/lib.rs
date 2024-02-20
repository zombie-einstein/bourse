mod order_book;
mod step_sim;
mod types;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "core")]
fn core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<order_book::OrderBook>()?;
    m.add_class::<step_sim::StepEnv>()?;
    Ok(())
}
