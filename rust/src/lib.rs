mod order_book;
mod step_sim;
mod step_sim_numpy;
mod types;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "core")]
fn core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<order_book::OrderBook>()?;
    m.add_class::<step_sim::StepEnv>()?;
    m.add_class::<step_sim_numpy::StepEnvNumpy>()?;
    m.add_function(wrap_pyfunction!(order_book::order_book_from_json, m)?)?;
    Ok(())
}
