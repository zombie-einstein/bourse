Simulation Example
==================

Here we will demonstrate a simple simulation
where agents randomly place orders.

First we import Bourse

.. testcode:: random_example

   import bourse


then define an agent class

.. testcode:: random_example

   class RandomAgent:
       def __init__(self, i, price_range):
           self.i = i
           self.price_range = price_range
           self.order_id = None

       def update(self, rng, env):
           # Place an order if one is not live
           if self.order_id is None:
               price = rng.integers(*self.price_range)
               side = bool(rng.choice([True, False]))
               env.place_order(side, 10, self.i, price=price)
           # Cancel live order
           else:
               env.cancel_order(self.order_id)
               self.order_id = None

For an agent to be part of a simulation it should
define an ``update`` method that takes a
:py:class:`numpy.random.Generator` and
:py:class:`bourse.core.StepEnv` as arguments.

In this example an agent places a random order if it
does not have an existing one, and otherwise attempts to
cancel its existing order.

We then initialise an environment and set of agents

.. testcode:: random_example

   seed = 101

   agents = [RandomAgent(i, (10, 100)) for i in range(50)]
   env = bourse.core.StepEnv(seed, 0, 1, 100_000)

We can then use :py:meth:`bourse.step_sim.run` to run the
simulation

.. testcode:: random_example

   n_steps = 50

   market_data = bourse.step_sim.run(env, agents, n_steps, seed)

``market_data`` is a dictionary of Numpy arrays containing market
data recorded over the course of the simulation.
