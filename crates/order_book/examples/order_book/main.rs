use bourse_book::{types, OrderBook};

fn main() {
    let mut book = OrderBook::new(0, 1, true);

    book.create_and_place_order(types::Side::Ask, 20, 0, Some(60));
    book.create_and_place_order(types::Side::Ask, 20, 0, Some(65));
    book.create_and_place_order(types::Side::Bid, 10, 0, Some(50));
    book.create_and_place_order(types::Side::Bid, 10, 0, Some(45));

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
