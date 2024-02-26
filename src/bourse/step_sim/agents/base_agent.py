"""
Base discrete event agent pattern
"""
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
