import numpy as np

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
