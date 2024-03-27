Orderbook
---------

An orderbook is initialised with a start time
(this is the time used to record events) and a
tick-size

.. testcode:: book_usage

   import bourse

   start_time = 0
   tick_size = 1

   book = bourse.core.OrderBook(start_time, tick_size)

The state of the orderbook an then be directly
updated, for example placing a limit bid order

.. testcode:: book_usage

   order_vol = 10
   trader_id = 101
   order_id = book.place_order(
       True, order_vol, trader_id, price=50
   )

or cancelling the same order

.. testcode:: book_usage

   book.cancel_order(order_id)

When directly interacting with the orderbook
updates are immediately applied and the state
of the market updated.

The orderbook also tracks updates, for example
trades executed on the order book can be
retrieved with

.. testcode:: book_usage

   trades = book.get_trades()
   # Convert trade data to a dataframe
   trade_df = bourse.data_processing.trades_to_dataframe(
       trades
   )

The state of the order book can be written to a JSON
file using :py:meth:`bourse.core.OrderBook.save_json_snapshot`,
the same snapshot can then be used to initialise an
orderbook using :py:meth:`bourse.core.order_book_from_json`

.. code-block:: python

   # Save order book state to foo.json
   book.save_json_snapshot("foo.json")

   # Create a new order book with state from the snapshot
   loaded_book = bourse.core.order_book_from_json("foo.json")

See :py:class:`bourse.core.OrderBook`
for details of the full order book API.
