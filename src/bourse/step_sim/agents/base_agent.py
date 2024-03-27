"""
Base discrete event agent pattern
"""
import typing

import numpy as np

from bourse import core


class BaseAgent:
    """
    Base discrete event agent
    """

    def update(self, rng: np.random.Generator, env: core.StepEnv):
        """
        Update the state of the agent and submit market instructions

        Update function called at each step of the simulation,
        should update the state of the agents(s) and submit
        and instructions to the environment.

        Parameters
        ----------
        rng: numpy.random.Generator
            Numpy random generator.
        env: bourse.core.StepEnv
            Discrete event simulation environment.
        """
        raise NotImplementedError


InstructionArrays = typing.Tuple[
    np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray
]


class BaseNumpyAgent:
    """
    Base discrete event agent using Numpy data

    Examples
    --------

    .. testcode:: base_numpy_agent

       import numpy as np
       from bourse.step_sim.agents import BaseNumpyAgent

       class Agent(BaseNumpyAgent):
           # This agent just gets the current touch prices
           #  and places an order either side of the spread
           def update(self, rng, level_2_data):
               bid, ask = level_2_data[1], level_2_data[2]

               return (
                   np.array([1, 1], dtype=np.uint32),
                   np.array([True, False]),
                   np.array([10, 20], dtype=np.uint32),
                   np.array([101, 202], dtype=np.uint32),
                   np.array([bid, ask], dtype=np.uint32),
                   np.array([0, 0], dtype=np.uint64),
               )
    """

    def update(
        self, rng: np.random.Generator, level_2_data: np.ndarray
    ) -> InstructionArrays:
        """
        Update the state of the agent and return new market instructions

        Update function called at each step of the simulation,
        should update the state of the agents(s) and return Numpy arrays
        represent instructions to submit to the market.

        Parameters
        ----------
        rng: numpy.random.Generator
            Numpy random generator.
        level_2_data: np.ndarray
            Numpy array representing the current state of the market.
            Contains the following values at positions:

            - 0: Trade volume (in the last step)
            - 1: Bid touch price
            - 2: Ask touch price
            - 3: Bid total volume
            - 4: Ask total volume

            the following 40 values are data for each
            of the 10 price level below/above the touch

            - Bid volume at level
            - Number of buy orders at level
            - Ask volume at level
            - Number of sell orders at level

        Returns
        -------
        tuple
            Tuple of numpy arrays containing:

            - Instruction type, an integer representing

              - ``0``: No change/null instruction
              - ``1``: New order
              - ``2``: Cancel order

            - Order sides (as bool, ``True`` for bid side) (used for new orders)
            - Order volumes (used for new orders)
            - Trader ids (used for new orders)
            - Order prices (used for new orders)
            - Order id (used for cancellations)
        """
        raise NotImplementedError
