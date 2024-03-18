import numpy as np
import pytest

import bourse


def test_submit_limit_orders_numpy():

    env = bourse.core.StepEnv(101, 0, 1, 100_000)

    sides = np.array([True, True, True, False, False, False])
    vols = np.array([10, 11, 12, 10, 11, 12], dtype=np.uint32)
    ids = np.array([1, 1, 1, 2, 2, 2], dtype=np.uint32)
    prices = np.array([20, 20, 19, 22, 22, 23], dtype=np.uint32)

    env.submit_limit_order_array((sides, vols, ids, prices))

    env.step()

    assert env.bid_ask == (20, 22)
    assert env.best_bid_vol_and_orders == (21, 2)
    assert env.best_ask_vol_and_orders == (21, 2)

    assert np.array_equal(
        env.level_1_data_array(),
        np.array([20, 22, 33, 33, 21, 2, 21, 2], dtype=np.uint32),
    )

    l2_data = env.level_2_data_array()

    assert l2_data.shape == (44,)
    assert np.array_equal(
        l2_data[:12],
        np.array([20, 22, 33, 33, 21, 2, 21, 2, 12, 1, 12, 1], dtype=np.uint32),
    )
    assert np.array_equal(l2_data[12:], np.zeros(32, dtype=np.uint32))


def test_raise_from_bad_order():

    env = bourse.core.StepEnv(101, 0, 2, 100_000)

    sides = np.array([True, True])
    vols = np.array([10, 11], dtype=np.uint32)
    ids = np.array([1, 1], dtype=np.uint32)
    prices = np.array([20, 21], dtype=np.uint32)

    with pytest.raises(BaseException):
        env.submit_limit_order_array((sides, vols, ids, prices))


def test_cancel_orders_from_array():

    env = bourse.core.StepEnv(101, 0, 1, 100_000)

    sides = np.array([True, True, True, False, False, False])
    vols = np.array([10, 11, 12, 10, 11, 12], dtype=np.uint32)
    ids = np.array([1, 1, 1, 2, 2, 2], dtype=np.uint32)
    prices = np.array([20, 20, 19, 22, 22, 23], dtype=np.uint32)

    env.submit_limit_order_array((sides, vols, ids, prices))

    env.step()

    env.submit_cancel_order_array(np.array([0, 1, 3, 4], dtype=np.uint64))

    env.step()

    assert env.bid_ask == (19, 23)
    assert env.best_bid_vol_and_orders == (12, 1)
    assert env.best_ask_vol_and_orders == (12, 1)


def test_numpy_random_agent():

    env = bourse.core.StepEnv(101, 0, 1, 100_000)
    agents = bourse.step_sim.agents.NumpyRandomAgents(20, (10, 60), (10, 20), 2)
    rng = np.random.default_rng(101)

    orders, cancellations = agents.update(rng, None)

    env.submit_limit_order_array(orders)
    env.submit_cancel_order_array(cancellations)
