Simulation Example
==================

Here we will demonstrate a simple simulation
where agents randomly place orders.

We will first define the agent class

.. code-block:: python

   class RandomAgent:
       def __init__(self, i, price_range):
           self.i = i
           self.price_range = price_range
           self.order_id = None

       def update(self, rng, env):
           # Place an order if not one live
           if self.order_id is None:
               price = rng.integers(*self.price_range)
               side = bool(rng.random.choice([True, False]))
               env.place_order(side, 10, self.i, price=price)
           # Cancel live order
           else:
               env.cancel_order(self.order_id)
               self.order_id = None

In this example an agent places a randomly order if it
does not have an existing one, and otherwise attempts to
cancel it's current order. Simulation agents need to define
an ``update`` function that takes a reference to a
Numpy random generator and a simulation environment.

We then initialise an environment and set of agents

.. code-block:: python

   seed = 101
   n_steps = 200

   agents = [RandomAgent(i, (10, 100)) for i in range(100)]
   env = bourse.core.StepEnv(seed, 0, 100_000)

We can then use :py:meth:`bourse.step_sim.run` to run the
simulation

.. code-block:: python

   market_data = bourse.step_sim.run(env, agents, n_steps, seed)

``market_data`` is a dictionary of Numpy arrays containing market
data recorded over the course of the simulation.
