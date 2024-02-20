import bourse


def test_order_book_init():
    ob = bourse.core.OrderBook(0)

    assert ob.bid_ask() == (0, bourse.MAX_PRICE)
    assert ob.bid_vol() == 0
    assert ob.ask_vol() == 0
    assert ob.best_bid_vol() == 0
    assert ob.best_ask_vol() == 0


def test_place_order():
    ob = bourse.core.OrderBook(0)

    ob.place_order(True, 10, 11, price=50)
    ob.place_order(False, 20, 12, price=60)

    assert ob.bid_ask() == (50, 60)
    assert ob.bid_vol() == 10
    assert ob.ask_vol() == 20
    assert ob.best_bid_vol() == 10
    assert ob.best_ask_vol() == 20

    ob.place_order(True, 10, 11, price=55)
    ob.place_order(False, 20, 12, price=65)

    assert ob.bid_ask() == (55, 60)
    assert ob.bid_vol() == 20
    assert ob.ask_vol() == 40
    assert ob.best_bid_vol() == 10
    assert ob.best_ask_vol() == 20


def test_cancel_order():
    ob = bourse.core.OrderBook(0)

    id_0 = ob.place_order(True, 10, 11, price=50)
    id_1 = ob.place_order(False, 20, 12, price=60)
    id_2 = ob.place_order(True, 10, 11, price=55)
    id_3 = ob.place_order(False, 20, 12, price=65)

    ob.cancel_order(id_2)
    ob.cancel_order(id_3)

    assert ob.order_status(id_2) == 3
    assert ob.order_status(id_3) == 3

    assert ob.bid_ask() == (50, 60)
    assert ob.bid_vol() == 10
    assert ob.ask_vol() == 20
    assert ob.best_bid_vol() == 10
    assert ob.best_ask_vol() == 20

    ob.cancel_order(id_0)
    ob.cancel_order(id_1)

    assert ob.order_status(id_0) == 3
    assert ob.order_status(id_1) == 3

    assert ob.bid_ask() == (0, bourse.MAX_PRICE)
    assert ob.bid_vol() == 0
    assert ob.ask_vol() == 0
    assert ob.best_bid_vol() == 0
    assert ob.best_ask_vol() == 0


def test_trades():
    ob = bourse.core.OrderBook(0)

    _ = ob.place_order(True, 10, 11, price=50)
    id_1 = ob.place_order(False, 20, 12, price=60)
    id_2 = ob.place_order(True, 10, 11, price=55)
    id_3 = ob.place_order(False, 20, 12, price=65)

    ob.set_time(10)

    # Place core order
    id_4 = ob.place_order(True, 30, 11)

    assert ob.order_status(id_4) == 2
    assert ob.order_status(id_1) == 2

    assert ob.bid_ask() == (55, 65)
    assert ob.bid_vol() == 20
    assert ob.ask_vol() == 10

    ob.set_time(20)

    # Place limit order
    id_5 = ob.place_order(False, 20, 12, price=55)

    assert ob.order_status(id_5) == 1
    assert ob.order_status(id_2) == 2

    assert ob.bid_ask() == (50, 55)
    assert ob.bid_vol() == 10
    assert ob.ask_vol() == 20

    trades = ob.get_trades()
    trade_df = bourse.data_processing.trades_to_dataframe(trades)

    assert len(trade_df) == 3
    assert list(trade_df["time"]) == [10, 10, 20]
    assert list(trade_df["price"]) == [60, 65, 55]
    assert list(trade_df["vol"]) == [20, 10, 10]
    assert list(trade_df["active_id"]) == [id_4, id_4, id_5]
    assert list(trade_df["passive_id"]) == [id_1, id_3, id_2]


def test_mod_order_volume():

    ob = bourse.core.OrderBook(0)

    _ = ob.place_order(True, 10, 11, price=50)
    id_1 = ob.place_order(True, 10, 11, price=55)
    id_2 = ob.place_order(False, 20, 12, price=65)
    _ = ob.place_order(False, 20, 12, price=60)

    ob.modify_order(id_1, new_vol=5)
    ob.modify_order(id_2, new_vol=10)

    assert ob.bid_ask() == (55, 60)
    assert ob.bid_vol() == 15
    assert ob.ask_vol() == 30
    assert ob.best_bid_vol() == 5
    assert ob.best_ask_vol() == 20


def test_modify_order():

    ob = bourse.core.OrderBook(0)

    a = ob.place_order(True, 10, 11, price=50)
    _ = ob.place_order(False, 30, 11, price=60)
    _ = ob.modify_order(a, new_price=45, new_vol=20)

    assert ob.bid_ask() == (45, 60)
    assert ob.bid_vol() == 20
    assert ob.ask_vol() == 30

    assert ob.order_status(a) == 1


def test_get_orders():

    ob = bourse.core.OrderBook(0)

    ob.place_order(True, 10, 11, price=50)
    ob.place_order(False, 20, 12, price=60)
    ob.place_order(True, 10, 11, price=55)
    ob.place_order(False, 20, 12, price=65)

    orders = ob.get_orders()
    orders_df = bourse.data_processing.orders_to_dataframe(orders)

    assert list(orders_df["side"]) == ["bid", "ask", "bid", "ask"]
    assert list(orders_df["vol"]) == [10, 20, 10, 20]
    assert list(orders_df["price"]) == [50, 60, 55, 65]
    assert list(orders_df["order_id"]) == [0, 1, 2, 3]
