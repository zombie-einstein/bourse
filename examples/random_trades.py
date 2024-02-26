import bourse
from bourse.step_sim.agents import RandomAgent


def run(seed: int, n_steps: int, n_agents: int):

    agents = [RandomAgent(i, 0.5, (10, 100), (20, 50), 2) for i in range(n_agents)]
    env = bourse.core.StepEnv(seed, 0, 100_000)

    market_data = bourse.step_sim.run(env, agents, n_steps, seed)

    return market_data


if __name__ == "__main__":
    run(101, 200, 100)
