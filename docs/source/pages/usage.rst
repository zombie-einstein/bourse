Usage
=====

.. note::

   Units for prices and times in the core Rust library
   are represented by integers.

   They are not implemented with any intended unit
   size, so the choice is left to the modeller, e.g.
   prices could represent a fixed number of decimal
   places, or time units could be  nanoseconds or
   milliseconds dependent on the use-case.


Bourse allows Python users and programs to interact
with three core pieces of functionality
from the Rust package:

- :doc:`order_book_usage`

  A simulated orderbook that allow orders to be directly
  placed and modified, and tracks trade and order data.

- :doc:`discrete_event_usage`

  A discrete event simulation environment that
  allows Python agents to submit trade
  instructions, with functionality to update
  simulation state and track simulation data.

- :doc:`numpy_discrete_event_usage`

  A discrete event simulation environment that can
  receive instructions and return data as Numpy arrays.

.. toctree::
   :hidden:

   order_book_usage
   discrete_event_usage
   numpy_discrete_event_usage
