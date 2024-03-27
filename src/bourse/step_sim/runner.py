"""
Discrete event simulation runner
"""
import typing

import numpy as np
import tqdm

import bourse


def run(
    env: typing.Union[bourse.core.StepEnv, bourse.core.StepEnvNumpy],
    agents: typing.Iterable,
    n_steps: int,
    seed: int,
    show_progress: bool = True,
    use_numpy: bool = False,
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
       env = env = bourse.core.StepEnv(0, 0, 2, 1000)

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
        with the simulation environment, or if ``use_numpy``
        is ``True`` then agents should return a tuple
        of Numpy array instructions. See
        :py:class:`bourse.step_sim.agents.base_agent.BaseAgent`
        and
        :py:class:`bourse.step_sim.agents.base_agent.BaseNumpyAgent`
        for more details.
    n_steps: int
        Number of simulation steps to run.
    seed: int
        Random seed.
    show_progress: bool, optional
        If ``True`` a progress bar will be displayed,
        default ``True``
    use_numpy: bool, optional
        If ``True`` use numpy api to for market state and
        to submit market instructions. Default ``False``

    Returns
    -------
    dict
        Dictionary containing level 2 market data with keys:

        - ``bid_price``: Bid price at each step
        - ``ask_price``: Ask price at each step
        - ``bid_vol``: Total bid volume at each step
        - ``ask_vol``: Total ask volume at each step
        - ``trade_vol``: Trade volume each step
        - ``bid_vol_<N>``: Bid volume at top 10 levels at each step
        - ``ask_vol_<N>``: Ask volume at top 10 levels at each step
        - ``n_bid_<N>``: Number of bid orders at top 10 levels at each step
        - ``n_ask_<N>``: Number of ask orders at top 10 levels at each step
    """

    if use_numpy:
        assert isinstance(env, bourse.core.StepEnvNumpy)
        assert all(
            [isinstance(a, bourse.step_sim.agents.BaseNumpyAgent) for a in agents]
        ), "Agents should implement bourse.step_sim.agents.BaseNumpyAgent"
    else:
        assert isinstance(env, bourse.core.StepEnv)
        assert all(
            [isinstance(a, bourse.step_sim.agents.BaseAgent) for a in agents]
        ), "Agents should implement bourse.step_sim.agents.BaseAgent"

    rng = np.random.default_rng(seed)
    it = tqdm.trange(n_steps, disable=not show_progress)

    if use_numpy:
        for _ in it:

            level_2_data = env.level_2_data()

            for agent in agents:
                instructions = agent.update(rng, level_2_data)
                env.submit_instructions(instructions)

            env.step()
    else:
        for _ in it:
            for agent in agents:
                agent.update(rng, env)

            env.step()

    return env.get_market_data()
