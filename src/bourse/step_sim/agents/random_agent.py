"""
Agent that places random uniformly sampled orders
"""
import typing

import numpy as np

from bourse import core
from bourse.step_sim.agents import BaseAgent, BaseNumpyAgent, InstructionArrays


class RandomAgent(BaseAgent):
    """
    Agent that place randomly sampled orders

    Agent that places orders on a random side with
    random volume and price within given ranges, where
    values are sampled from uniform distributions.

    Warnings
    --------
    The behaviour of this agent is not intended to
    represent any 'realistic' behaviour, so should
    really be used for testing or benchmarking.
    """

    def __init__(
        self,
        i: int,
        activity_rate: float,
        tick_range: typing.Tuple[int, int],
        vol_range: typing.Tuple[int, int],
        tick_size: int,
    ):
        """
        Initialise a RandomAgent

        Parameters
        ----------
        i: int
            Id of the agent
        activity_rate: float
            Activity rate of the agent(i.e. the
            probability the agent is active each step).
        tick_range: tuple[int, int]
            Tick range to sample from.
        vol_range: tuple[int, int]
            Volume range to sample from.
        tick_size: int
            Size of a market tick
        """
        self.i = i
        self.activity_rate = activity_rate
        self.tick_range = tick_range
        self.vol_range = vol_range
        self.tick_size = tick_size
        self.order_id = None

    def update(self, rng: np.random.Generator, env: core.StepEnv):
        """
        Cancel a live order or place a new random one

        If the agent is active this step it will:

        - If it has an active order on the market it
          will submit an instruction to cancel it.
        - If it has no active orders it will place a new
          order on a random side with random volume and price.

        Parameters
        ----------
        rng: numpy.random.Generator
            Numpy random generator.
        env: bourse.core.StepEnv
            Discrete event simulation environment.
        """
        p = rng.random()

        if p < self.activity_rate:
            # If an order is live then cancel it
            if self.order_id is not None and env.order_status(self.order_id) == 1:
                env.cancel_order(self.order_id)
                self.order_id = None
            # Otherwise place a new random order
            else:
                tick = rng.integers(*self.tick_range)
                vol = rng.integers(*self.vol_range)
                side = bool(rng.choice([True, False]))
                self.order_id = env.place_order(
                    side, vol, self.i, price=tick * self.tick_size
                )


class NumpyRandomAgents(BaseNumpyAgent):
    """
    Simple agent set that places random orders via Numpy arrays

    Agents that place random orders each step of the simulation, new
    orders are returned as a tuple of Numpy arrays. These orders
    can then be submitted to a discrete event environment using
    :py:meth:`bourse.core.StepEnv.submit_limit_order_array`.

    This agent type is designed to represent a group of agents all
    placing individual orders at each step (rather than a single agent).
    """

    def __init__(
        self,
        n_agents: int,
        tick_range: typing.Tuple[int, int],
        vol_range: typing.Tuple[int, int],
        tick_size: int,
    ):
        """
        Initialise NumpyRandomAgents

        Parameters
        ----------
        n_agents: int
            Number of agents in the set
        tick_range: tuple[int, int]
            Tick range to sample from.
        vol_range: tuple[int, int]
            Volume range to sample from.
        tick_size: int
            Size of a market tick
        """
        self.n_agents = n_agents
        self.tick_range = tick_range
        self.vol_range = vol_range
        self.tick_size = tick_size

    def update(
        self, rng: np.random.Generator, level_2_data: np.ndarray
    ) -> InstructionArrays:
        """
        Update the agents, sampling new orders to place

        Parameters
        ----------
        rng: numpy.random.Generator
            Numpy random generator.
        level_2_data: bourse.core.StepEnv
            Level-2 market data

        Returns
        -------
        tuple
            Tuple containing new order instructions
        """
        sides = rng.choice([True, False], size=self.n_agents).astype(bool)
        vols = rng.integers(*self.tick_range, size=self.n_agents, dtype=np.uint32)
        ids = np.arange(self.n_agents, dtype=np.uint32)
        prices = (
            rng.integers(*self.tick_range, size=self.n_agents, dtype=np.uint32)
            * self.tick_size
        )

        return (
            np.ones(self.n_agents, dtype=np.uint32),
            sides,
            vols,
            ids,
            prices,
            np.zeros(self.n_agents, dtype=np.uint64),
        )
