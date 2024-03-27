import numpy as np
import pytest

import bourse


def test_submit_limit_orders_numpy():

    env = bourse.core.StepEnvNumpy(101, 0, 1, 100_000)

    sides = np.array([True, True, True, False, False, False])
    vols = np.array([10, 11, 12, 10, 11, 12], dtype=np.uint32)
    ids = np.array([1, 1, 1, 2, 2, 2], dtype=np.uint32)
    prices = np.array([20, 20, 19, 22, 22, 23], dtype=np.uint32)

    ids = env.submit_limit_orders((sides, vols, ids, prices))

    env.step()

    assert np.array_equal(ids, np.arange(6))

    assert np.array_equal(
        env.level_1_data(),
        np.array([0, 20, 22, 33, 33, 21, 2, 21, 2], dtype=np.uint32),
    )

    l2_data = env.level_2_data()

    assert l2_data.shape == (45,)
    assert np.array_equal(
        l2_data[:13],
        np.array([0, 20, 22, 33, 33, 21, 2, 21, 2, 12, 1, 12, 1], dtype=np.uint32),
    )
    assert np.array_equal(l2_data[13:], np.zeros(32, dtype=np.uint32))


def test_raise_from_bad_order():

    env = bourse.core.StepEnvNumpy(101, 0, 2, 100_000)

    sides = np.array([True, True])
    vols = np.array([10, 11], dtype=np.uint32)
    ids = np.array([1, 1], dtype=np.uint32)
    prices = np.array([20, 21], dtype=np.uint32)

    with pytest.raises(BaseException):
        env.submit_limit_orders((sides, vols, ids, prices))


def test_cancel_orders_from_array():

    env = bourse.core.StepEnvNumpy(101, 0, 1, 100_000)

    sides = np.array([True, True, True, False, False, False])
    vols = np.array([10, 11, 12, 10, 11, 12], dtype=np.uint32)
    ids = np.array([1, 1, 1, 2, 2, 2], dtype=np.uint32)
    prices = np.array([20, 20, 19, 22, 22, 23], dtype=np.uint32)

    env.submit_limit_orders((sides, vols, ids, prices))

    env.step()

    env.submit_cancellations(np.array([0, 1, 3, 4], dtype=np.uint64))

    env.step()

    level_1_data = env.level_1_data()

    assert (level_1_data[1], level_1_data[2]) == (19, 23)
    assert (level_1_data[5], level_1_data[6]) == (12, 1)
    assert (level_1_data[7], level_1_data[8]) == (12, 1)


def test_numpy_random_agent():

    env = bourse.core.StepEnvNumpy(101, 0, 1, 100_000)
    agents = bourse.step_sim.agents.NumpyRandomAgents(20, (10, 60), (10, 20), 2)
    rng = np.random.default_rng(101)

    instructions = agents.update(rng, env.level_2_data())

    env.submit_instructions(instructions)
