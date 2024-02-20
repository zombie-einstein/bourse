Usage
=====

Bourse allows Python users to interact
with two core pieces of functionality
from the Rust package:

- An orderbook that allow orders to be directly
  placed and modified.
- A discrete event simulation environment that
  allows Python agents to submit trade
  instructions with functionality to update
  simulation state and track simulation data.

Orderbook
---------

An orderbook is initialised with a start time
(this is the time used to record events)

.. code-block:: python

   import bourse

   book = bourse.core.OrderBook(0)

The state of the orderbook an then be directly
updated, for example placing a limit bid order

.. code-block:: python

   order_vol = 10
   trader_id = 101
   order_id = book.place_order(
       True, order_vol, trader_id, price=50
   )

or cancelling the same order

.. code-block:: python

   book.cancel_order(order_id)

When directly interacting with the orderbook
updates are immediately applied and the state
of the market updated.

The orderbook also tracks updates, for example
trades executed on the order book can be
retrieved with

.. code-block:: python

   trades = book.get_trades()
   # Convert trade data to a dataframe
   trade_df = bourse.data_processing.trades_to_dataframe(
       trades
   )

See :py:class:`bourse.core.OrderBook`
for details of the orderbook API.

Simulation Environment
----------------------

A simulation environment can be initialised from
a random seed, start-time, and step-size (i.e. how
long in time each simulated step is)

.. code-block:: python

   seed = 101
   step_size = 100_000
   env = bourse.core.StepEnv(seed, 0, step_size)

The state of the simulation is updated in discrete
steps, with transactions submitted to a queue to
be processed at the end of the step. For example
placing new orders

.. code-block:: python

   order_id_a = env.place_order(False, 100, 101, price=60)
   order_id_b = env.place_order(True, 100, 101, price=70)

To actually update the state of the simulation we call
:py:meth:`bourse.core.StepEnv.step` which shuffles and
processes the queued instructions. Each step also increments
time to correctly order transactions.

The simulation environment also tracks market data for each
step, for example bid-ask prices can be retrieved using

.. code-block:: python

   bid_prices, ask_prices = env.get_prices()

See :py:class:`bourse.core.StepEnv` for full details
of the environment API.

:py:meth:`bourse.step_sim.run` is a utility for running a
simulation from an environment and set of agents. See
:ref:`Simulation Example` for a full simulation example.
