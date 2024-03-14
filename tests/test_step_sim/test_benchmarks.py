import numpy as np
import pytest

import bourse
from bourse.step_sim.agents import NumpyRandomAgents, RandomAgent

SEED = 101
N_AGENTS = 200
TiCK_SIZE = 2


@pytest.fixture
def env():
    return bourse.core.StepEnv(SEED, 0, TiCK_SIZE, 100_000)


@pytest.fixture
def agents():
    return [
        RandomAgent(i, 0.5, (10, 100), (20, 50), TiCK_SIZE) for i in range(N_AGENTS)
    ]


@pytest.fixture
def numpy_agents():
    return [
        NumpyRandomAgents(N_AGENTS, (10, 100), (20, 50), TiCK_SIZE),
    ]


def run_sim(n_steps, seed, e, a):
    return bourse.step_sim.run(e, a, n_steps, seed)


def run_numpy_sim(n_steps, seed, e, a):

    rng = np.random.default_rng(seed)

    for _ in range(n_steps):
        for agent in a:
            orders, cancels = agent.update(rng, e)
            e.submit_limit_order_array(orders)
            e.submit_cancel_order_array(cancels)


def test_simulation_benchmark(benchmark, env, agents):
    n_steps = 100
    seed = 101
    benchmark(run_sim, n_steps, seed, env, agents)


def test_numpy_simulation_benchmark(benchmark, env, numpy_agents):
    n_steps = 100
    seed = 101
    benchmark(run_numpy_sim, n_steps, seed, env, numpy_agents)
