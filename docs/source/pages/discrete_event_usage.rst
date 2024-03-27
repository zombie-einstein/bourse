Discrete Event Simulation Environment
-------------------------------------

A discrete event simulation environment can be initialised from
a random seed, start-time, tick-size, and step-size (i.e. how
long in time each simulated step is)

.. testcode:: sim_usage

   import bourse

   seed = 101
   start_time = 0
   tick_size = 2
   step_size = 100_000

   env = bourse.core.StepEnv(seed, start_time, tick_size, step_size)

The state of the simulation is updated in discrete
steps, with transactions submitted to a queue to
be processed at the end of the step. For example
placing new orders

.. testcode:: sim_usage

   order_id_a = env.place_order(False, 100, 101, price=60)
   order_id_b = env.place_order(True, 100, 101, price=70)

To actually update the state of the simulation we call
:py:meth:`bourse.core.StepEnv.step` which shuffles and
processes the queued instructions. Each step also increments
time to correctly order transactions.

The simulation environment also tracks market data for each
step, for example bid-ask prices can be retrieved using

.. testcode:: sim_usage

   bid_prices, ask_prices = env.get_prices()

the full level 2 data (price and volumes along with volumes
and number of orders at top 10 levels) records can be
retrieved with

.. testcode:: sim_usage

   level_2_data = env.get_market_data()

See :py:class:`bourse.core.StepEnv` for full details
of the environment API.

:py:meth:`bourse.step_sim.run` is a utility for running a
simulation from an environment and set of agents. See
:ref:`Simulation Example` for a full simulation example.
