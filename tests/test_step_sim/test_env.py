import numpy as np

import bourse


def test_step_sim_env():

    env = bourse.core.StepEnv(101, 0, 1, 100_000)

    env.place_order(True, 100, 101, price=50)
    env.place_order(False, 100, 101, price=60)

    env.step()
    assert env.bid_ask == (50, 60)
    assert env.ask_vol == 100
    assert env.bid_vol == 100
    assert env.time == 100_000

    env.place_order(True, 100, 101, price=55)
    env.place_order(False, 100, 101, price=65)

    env.step()
    assert env.bid_ask == (55, 60)
    assert env.ask_vol == 200
    assert env.bid_vol == 200
    assert env.time == 200_000

    env.place_order(True, 150, 101)
    env.step()

    assert env.bid_ask == (55, 65)
    assert env.ask_vol == 50
    assert env.bid_vol == 200
    assert env.time == 300_000

    env.step()

    bids, asks = env.get_prices()

    assert isinstance(bids, np.ndarray)
    assert isinstance(asks, np.ndarray)

    assert np.array_equal(bids, np.array([50, 55, 55, 55]))
    assert np.array_equal(asks, np.array([60, 60, 65, 65]))

    bid_vols, ask_vols = env.get_volumes()

    assert isinstance(bid_vols, np.ndarray)
    assert isinstance(ask_vols, np.ndarray)

    assert np.array_equal(bid_vols, np.array([100, 200, 200, 200]))
    assert np.array_equal(ask_vols, np.array([100, 200, 50, 50]))

    bid_touch_vols, ask_touch_vols = env.get_touch_volumes()

    assert isinstance(bid_touch_vols, np.ndarray)
    assert isinstance(ask_touch_vols, np.ndarray)

    assert np.array_equal(bid_touch_vols, np.array([100, 100, 100, 100]))
    assert np.array_equal(ask_touch_vols, np.array([100, 100, 50, 50]))

    bid_touch_counts, ask_touch_counts = env.get_touch_order_counts()
    assert isinstance(bid_touch_counts, np.ndarray)
    assert isinstance(ask_touch_counts, np.ndarray)

    assert np.array_equal(bid_touch_counts, np.array([1, 1, 1, 1]))
    assert np.array_equal(ask_touch_counts, np.array([1, 1, 1, 1]))

    trade_vols = env.get_trade_volumes()

    assert isinstance(trade_vols, np.ndarray)
    assert np.array_equal(trade_vols, np.array([0, 0, 150, 0]))

    core_data = env.get_market_data()

    assert isinstance(core_data, dict)

    expected_keys = {
        "bid_price",
        "ask_price",
        "bid_vol",
        "ask_vol",
        "bid_touch_vol",
        "ask_touch_vol",
        "bid_touch_order_count",
        "ask_touch_order_count",
        "trade_vol",
    }

    assert set(core_data.keys()) == expected_keys

    assert np.array_equal(core_data["bid_price"], np.array([50, 55, 55, 55]))
    assert np.array_equal(core_data["ask_price"], np.array([60, 60, 65, 65]))

    assert np.array_equal(core_data["bid_vol"], np.array([100, 200, 200, 200]))
    assert np.array_equal(core_data["ask_vol"], np.array([100, 200, 50, 50]))

    assert np.array_equal(core_data["bid_touch_vol"], np.array([100, 100, 100, 100]))
    assert np.array_equal(core_data["ask_touch_vol"], np.array([100, 100, 50, 50]))

    assert np.array_equal(core_data["bid_touch_order_count"], np.array([1, 1, 1, 1]))
    assert np.array_equal(core_data["ask_touch_order_count"], np.array([1, 1, 1, 1]))

    assert np.array_equal(core_data["trade_vol"], np.array([0, 0, 150, 0]))


def test_runner():
    class TestAgent:
        def __init__(self, side: bool, start_price: int):
            self.side = side
            self.start_price = start_price
            self.step = 0

        def update(self, _rng, env):
            if self.side:
                new_price = self.start_price + self.step
            else:
                new_price = self.start_price - self.step

            env.place_order(self.side, 10, 101, price=new_price)
            self.step += 1

    env = bourse.core.StepEnv(101, 0, 1, 100_000)
    agents = [TestAgent(True, 10), TestAgent(False, 50)]

    data = bourse.step_sim.run(env, agents, 10, 101)

    assert np.array_equal(data["bid_price"], 10 + np.arange(10))
    assert np.array_equal(data["ask_price"], 50 - np.arange(10))
    assert np.array_equal(data["bid_vol"], 10 * np.arange(1, 11))
    assert np.array_equal(data["ask_vol"], 10 * np.arange(1, 11))
    assert np.array_equal(data["bid_touch_vol"], 10 * np.ones(10))
    assert np.array_equal(data["ask_touch_vol"], 10 * np.ones(10))
    assert np.array_equal(data["trade_vol"], np.zeros(10))
