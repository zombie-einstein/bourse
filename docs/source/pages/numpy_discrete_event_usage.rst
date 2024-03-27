Numpy Discrete Event Simulation Environment
-------------------------------------------

This environment allows market state and market instructions
to be returned/submitted as Numpy arrays. This has the
potential for higher performance (over native Python) using
vectorisation (with some limitations on functionality) in
particular for ML and RL use-cases.

The simulation environment can be initialised from
a random seed, start-time, tick-size, and step-size (i.e. how
long in time each simulated step is)

.. testcode:: numpy_sim_usage

   import numpy as np
   import bourse

   seed = 101
   start_time = 0
   tick_size = 2
   step_size = 100_000

   env = bourse.core.StepEnvNumpy(seed, start_time, tick_size, step_size)

New order and cancellations can be submitted as Numpy arrays

.. testcode:: numpy_sim_usage

   new_orders = (
       # Order sides
       np.array([True, False]),
       # Order volumes
       np.array([10, 20], dtype=np.uint32),
       # Trader ids
       np.array([101, 202], dtype=np.uint32),
       # Order prices
       np.array([50, 60], dtype=np.uint32),
   )

   new_order_ids = env.submit_limit_orders(new_orders)

   # Update the environment state (placing orders)
   env.step()

   # Cancel the new orders
   env.submit_cancellations(new_order_ids)

Multiple instruction types can be submitted as a tuple of arrays:

- The instruction type where ``0 = no action``, ``1 = new-order``, and
  ``2 = cancellation``.
- Order sides (as bool, ``True`` for bid side) (used for new orders)
- Order volumes (used for new orders)
- Trader ids (used for new orders)
- Order prices (used for new orders)
- Order ids (used for cancellations)

.. note::

   Values that are not used for a given action (e.g. order-ids for
   new orders) are ignored, so can be set to an arbitrary default.

For example, if we want to submit one instruction with no change
and one new-order we could use:

.. testcode:: numpy_sim_usage

   instructions = (
       np.array([0, 1], dtype=np.uint32),
       np.array([True, True]),
       np.array([0, 20], dtype=np.uint32),
       np.array([0, 101], dtype=np.uint32),
       np.array([0, 50], dtype=np.uint32),
       np.array([0, 0], dtype=np.uint64)
   )

   new_order_ids = env.submit_instructions(instructions)

   env.step()

.. warning::

   This method currently only supports submitting limit
   orders and cancelling orders.

The state of the order book can be retrieved as an array
of values representing the current touch-prices, volumes and
volumes and orders at price levels

.. testcode:: numpy_sim_usage

   level_1_data = env.level_1_data()

   level_2_data = env.level_2_data()

where the level-1 data only contains the touch volume and
number of orders, and level-2 data contains the volume and
number of orders for the first 10 price levels from the touch.

See :py:class:`bourse.core.StepEnvNumpy` for full details
of the API.

Agents that interact with the Numpy API can implement
:py:class:`bourse.step_sim.agents.base_agent.BaseNumpyAgent` with an
``update`` method that takes a random number generator
and array representing the current level 2 data of the
order book (the current touch price, and volumes and orders
at the top 10 price levels). It should return a tuple of
arrays encoding market instructions, for example this
agent simply places new orders either side of the spread

.. testcode:: numpy_sim_usage

   from bourse.step_sim.agents import BaseNumpyAgent

   class Agent(BaseNumpyAgent):

      def update(self, rng, level_2_data):
          bid = max(level_2_data[1], 20)
          ask = min(level_2_data[2], 40)

          return (
             np.array([1, 1], dtype=np.uint32),
             np.array([True, False]),
             np.array([10, 20], dtype=np.uint32),
             np.array([101, 202], dtype=np.uint32),
             np.array([bid, ask], dtype=np.uint32),
             np.array([0, 0], dtype=np.uint64),
          )

These agents can be used in simulation by setting the
``use_numpy`` argument, and passing an array
of agents implementing :py:class:`bourse.step_sim.agents.base_agent.BaseNumpyAgent`,
for example

.. testcode:: numpy_sim_usage

   agents = [Agent()]

   n_steps = 50
   seed = 101

   market_data = bourse.step_sim.run(
      env, agents, n_steps, seed, use_numpy = True
   )
