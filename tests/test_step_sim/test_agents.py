import numpy as np

import bourse


def test_random_agent():
    env = bourse.core.StepEnv(101, 0, 1, 100_000)

    agent = bourse.step_sim.agents.RandomAgent(0, 1.0, (10, 20), (20, 30), 2)

    rng = np.random.default_rng(101)

    agent.update(rng, env)

    assert agent.order_id == 0

    env.step()

    assert env.order_status(0) == 1

    agent.update(rng, env)

    assert agent.order_id is None

    env.step()

    assert env.order_status(0) == 3

    agent.update(rng, env)

    assert agent.order_id == 1
