"""
Discrete event simulation runner
"""
import typing

import numpy as np
import tqdm

import bourse


def run(
    env: bourse.core.StepEnv,
    agents: typing.Iterable,
    n_steps: int,
    seed: int,
) -> typing.Dict[str, np.ndarray]:
    """
    Run a discrete event simulation for fixed number of steps

    Runs a discrete event simulation. Each step of the
    simulation agents submit transactions to the
    simulation environment. At the end of the transactions
    are randomly shuffled and process, updating the state of
    the market.

    Examples
    --------

    .. testsetup:: runner_docstring

       import bourse

       agents = []
       env = env = bourse.core.StepEnv(0, 0, 1000)

    .. testcode:: runner_docstring

       market_data = bourse.step_sim.run(
           env,     # Simulation environment
           agents,  # List of agents
           50,      # Number of steps
           101      # Random seed
       )

    Parameters
    ----------
    env: bourse.core.StepEnv
        Step updating simulation environment
    agents: list or tuple
        Iterable containing initialised agents. Agents
        should have an ``update`` method that interacts
        with the simulation environment.
    n_steps: int
        Number of simulation steps to run.
    seed: int
        Random seed.

    Returns
    -------
    dict
        Dictionary containing market data with keys:

        - ``bid_price``: Bid price at each step
        - ``ask_price``: Ask price at each step
        - ``bid_vol``: Total bid volume at each step
        - ``ask_vol``: Total ask volume at each step
        - ``trade_vol``: Trade volume each step
    """

    rng = np.random.default_rng(seed)

    for _ in tqdm.trange(n_steps):
        for agent in agents:
            agent.update(rng, env)

        env.step()

    return env.get_market_data()
