"""
Utilities to process data returned from Rust orderbook and simulation environments
"""

import typing

import pandas as pd


def trades_to_dataframe(trades: typing.List[typing.Tuple]) -> pd.DataFrame:
    """
    Convert trade data to a Pandas dataframe

    Convert trade data returned from

    - :py:meth:`bourse.core.OrderBook.get_trades`
    - :py:meth:`bourse.core.StepEnv.get_trades`

    into a Pandas dataframe.

    Parameters
    ----------
    trades: list[tuple]
        List of trade data.

    Returns
    -------
    pandas.DataFrame
        Pandas dataframe with columns:

        - ``time``: Trade time (in nano-seconds)
        - ``side``: Order-book side of the trade
        - ``vol``: Trade volume
        - ``active_id``: Order id of the aggressive order
        - ``passive_id``: Order id of the passive order

    Notes
    -----
    Numerical values are stored as unsigned types.
    """
    columns = ["time", "side", "price", "vol", "active_id", "passive_id"]

    df = pd.DataFrame.from_records(trades, columns=columns)
    df["side"] = df["side"].map({True: "bid", False: "ask"})

    return df


def orders_to_dataframe(order_history: typing.List[typing.Tuple]) -> pd.DataFrame:
    """
    Convert order data to a Pandas dataframe

    Converts order data returned from

    - :py:meth:`bourse.core.OrderBook.get_orders`
    - :py:meth:`bourse.core.StepEnv.get_orders`

    into a Pandas dataframe.

    Parameters
    ----------
    order_history: list[tuple]
        List of order data

    Returns
    -------
    pandas.DataFrame
        Pandas dataframe with columns:

        - ``side``: Side the order was placed.
        - ``status``: Status of the order.
        - ``arr_time``: Arrival time of the order.
        - ``end_time``: End time of the order.
        - ``vol``: Remaining volume of the order.
        - ``start_vol``: Original volume of the order.
        - ``price``: Price of the order (market orders
          will be listed as 0 or 2 :sup:`32` dependent
          on side).
        - ``trader_id``: Id of the agent that placed
          the order.
        - ``order_id``: Id of the order.

    Notes
    -----
    Numerical values are stored as unsigned types.
    """
    columns = [
        "side",
        "status",
        "arr time",
        "end_time",
        "vol",
        "start_vol",
        "price",
        "trader_id",
        "order_id",
    ]

    df = pd.DataFrame.from_records(order_history, columns=columns)
    df["side"] = df["side"].map({True: "bid", False: "ask"})
    df["status"] = df["status"].map(
        {0: "new", 1: "active", 2: "filled", 3: "cancelled", 4: "rejected"}
    )

    return df
