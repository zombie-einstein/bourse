Discrete Event Simulation Environment
-------------------------------------

A discrete event simulation environment can be initialised
with the parameters:

- *random seed*: A value used to seed the random number
  generator.
- *start time*: Time to initialise the orderbook
  with. The time field of the orderbook is used
  when recording events and trades.
- *tick-size*: Value between price-levels
  of the order book. Prices are represented by
  integers, so the tick-size can represent an
  arbitrary value (e.g. a fixed number of decimal
  places). A value of ``1`` can be used for simplicity,
  but different values can be used to represent
  assets with differing tick sizes.
- *step-size*: The (simulated) time between simulation
  steps, i.e. each simulation step the simulated time
  is incremented by this amount. Inside a step
  the time is incremented by ``1`` for each instruction,
  so this value should account for this, e.g. a step-size
  of ``10_000`` will allow for 10,000 instructions to
  be processed inside a single step.

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
new order instructions can be submitted to the queue
using

.. testcode:: sim_usage

   order_id_a = env.place_order(False, 100, 101, price=60)
   order_id_b = env.place_order(True, 100, 101, price=70)

To actually update the state of the simulation we call
:py:meth:`bourse.core.StepEnv.step` which shuffles and
processes the queued instructions. Each step also increments
time inside the step to correctly order transactions.

The simulation environment also tracks market data for each
step, for example bid-ask prices at each step of
the lifetime of the environment can be retrieved using

.. testcode:: sim_usage

   bid_prices, ask_prices = env.get_prices()

the full level 2 data (price and volumes along with volumes
and number of orders at top 10 levels) records can be
retrieved with

.. testcode:: sim_usage

   level_2_data = env.get_market_data()

See :py:class:`bourse.core.StepEnv` for full details
of the environment API. In addition :py:meth:`bourse.step_sim.run`
is a utility for running a simulation from an environment and
set of agents, see :ref:`Simulation Example` for an annotated
example of simulation code.
