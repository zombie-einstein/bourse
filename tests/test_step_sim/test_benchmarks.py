import pytest

import bourse
from bourse.step_sim.agents import RandomAgent

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


def run_sim(n_steps, seed, e, a):
    return bourse.step_sim.run(e, a, n_steps, seed)


def test_simulation_benchmark(benchmark, env, agents):
    n_steps = 100
    seed = 101
    benchmark(run_sim, n_steps, seed, env, agents)
