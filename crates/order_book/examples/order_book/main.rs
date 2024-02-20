use bourse_book::{types, OrderBook};

fn main() {
    let mut book = OrderBook::new(0, true);

    let id_a = book.create_order(types::Side::Ask, 20, 0, Some(60));
    let id_b = book.create_order(types::Side::Ask, 20, 0, Some(65));
    let id_c = book.create_order(types::Side::Bid, 10, 0, Some(50));
    let id_d = book.create_order(types::Side::Bid, 10, 0, Some(45));

    book.place_order(id_a);
    book.place_order(id_b);
    book.place_order(id_c);
    book.place_order(id_d);

    let (bid, ask) = book.bid_ask();

    println!(
        "bid-ask prices: \t{} {}\nbid-ask volumes:\t{} {}",
        bid,
        ask,
        book.bid_vol(),
        book.ask_vol()
    );

    let id_e = book.create_order(types::Side::Ask, 15, 99, None);

    book.set_time(10);
    book.place_order(id_e);

    println!("\nTrades\n------");

    for trade in book.get_trades().iter() {
        println!(
            "t: {}, side: {:?}, price: {}, vol: {}",
            trade.t, trade.side, trade.price, trade.vol
        );
    }

    println!("\nOrders\n------");

    for order in book.get_orders().iter() {
        println!(
            "id: {}, side: {:?}, status: {:?}",
            order.order_id, order.side, order.status
        );
    }
}
