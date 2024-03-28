Usage
=====

Bourse allows Python users/programs to interact
with three core pieces of functionality
from the Rust package:

- :doc:`order_book_usage`

  An orderbook that allow orders to be directly
  placed and modified, and tracks trade and order data.

- :doc:`discrete_event_usage`

  A discrete event simulation environment that
  allows Python agents to submit trade
  instructions, with functionality to update
  simulation state and track simulation data.

- :doc:`numpy_discrete_event_usage`

  A discrete event simulation environment can receive
  instructions and returns data as Numpy arrays.

.. toctree::
   :hidden:

   order_book_usage
   discrete_event_usage
   numpy_discrete_event_usage
