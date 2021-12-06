use costbasis::holding::Holding;
use costbasis::inventory::InventoryType;
use costbasis::realized::Realized;
use costbasis::transaction::Transaction;
use costbasis::unrealized::URealized;

#[test]
fn buy_only_initiate_unrealized() {
    let transaction = Transaction::from("2020-01-01,long,100.0,25.0");
    let results_ur = [URealized::from("2020-01-01,100.0,-2500.0")];
    let mut holding = Holding::default();
    let gains_r = holding.add_transaction(&transaction);
    assert_eq!(gains_r, vec!());
    assert_eq!(holding.inventory(), results_ur);

    let holding = Holding::new(&transaction);
    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn multiple_buy_unrealized() {
    let transactions = [
        Transaction::from("2020-01-01,long,100.0,25.0"),
        Transaction::from("2020-02-01,long,100.0,35.0"),
        Transaction::from("2020-03-01,long,100.0,30.0"),
    ];
    let results_ur = [
        URealized::from("2020-01-01,100.0,-2500.0"),
        URealized::from("2020-02-01,100.0,-3500.0"),
        URealized::from("2020-03-01,100.0,-3000.0"),
    ];
    let mut holding = Holding::default();
    let gains_r = holding.extend_transactions(&transactions);
    assert_eq!(gains_r, vec!());
    assert_eq!(holding.inventory(), results_ur);

    // one by one
    let mut holding = Holding::new(&transactions[0]);
    let gains2_r = holding.extend_transactions(&transactions[1..]);
    assert_eq!(gains2_r, vec!());
    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn buy_sell_only_realized() {
    let transactions = [
        Transaction::from("2020-01-01,long,100.0,25.0"),
        Transaction::from("2020-02-01,short,100.0,35.0"),
    ];
    let results_r = [Realized::from(
        "2020-02-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0",
    )];
    let mut holding = Holding::new(&transactions[0]);
    let gains_r = holding.add_transaction(&transactions[1]);
    assert_eq!(gains_r, results_r);
    assert_eq!(holding.inventory(), vec!());
}

#[test]
fn buy_sell_realized_and_unrealized_left() {
    let transactions = [
        Transaction::from("2020-01-01,long,200.0,25.0"),
        Transaction::from("2020-02-01,short,100.0,35.0"),
    ];
    let results_ur = vec![URealized::from("2020-01-01,100.0,-2500.0")];
    let results_r = [Realized::from(
        "2020-02-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0",
    )];
    let mut holding = Holding::new(&transactions[0]);
    let gains_r = holding.add_transaction(&transactions[1]);
    assert_eq!(gains_r, results_r);
    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn buy_sell_realized_and_unrealized() {
    let transactions = [
        Transaction::from("2020-01-01,long,100.0,25.0"),
        Transaction::from("2020-02-01,short,200.0,35.0"),
    ];
    let results_ur = vec![URealized::from("2020-02-01,-100.0,3500.0")];
    let results_r = [Realized::from(
        "2020-02-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0",
    )];
    let mut holding = Holding::new(&transactions[0]);
    let gains_r = holding.add_transaction(&transactions[1]);
    assert_eq!(gains_r, results_r);
    assert_eq!(holding.inventory(), results_ur);
    assert_eq!(holding.direction(), Some(InventoryType::Short));
}
#[test]
fn buy_sell_realized_and_unrealized_starting_with_multiple_unrealized() {
    let starting_ur = [
        URealized::from("2020-01-01,100.0,-2500.0"),
        URealized::from("2020-02-01,100.0,-2500.0"),
        URealized::from("2020-03-01,100.0,-2500.0"),
    ];
    let mut holding = Holding::from(&starting_ur[..]);

    let transaction = Transaction::from("2020-04-01,short,250.0,35.0");
    let results_ur = [URealized::from("2020-03-01,50.0,-1250.0")];
    let results_r = [
        Realized::from("2020-04-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-04-01,-100.0,3500.0,2020-02-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-04-01,-50.0,1750.0,2020-03-01,50.0,-1250.0,500.0"),
    ];

    let gains_r = holding.add_transaction(&transaction);
    assert_eq!(gains_r, results_r);
    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn open_close_more_than_once_zero_balance_twice() {
    let starting_ur = [
        URealized::from("2020-01-01,100.0,-2500.0"),
        URealized::from("2020-02-01,100.0,-2500.0"),
    ];
    let mut holding = Holding::from(&starting_ur[..]);
    let transactions = [
        Transaction::from("2020-04-01,short,200.0,35.0"),
        Transaction::from("2020-05-01,long,100.0,25.0"),
        Transaction::from("2020-06-01,short,100.0,35.0"),
    ];
    let results_r = [
        Realized::from("2020-04-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-04-01,-100.0,3500.0,2020-02-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-06-01,-100.0,3500.0,2020-05-01,100.0,-2500.0,1000.0"),
    ];

    let gains_r = holding.extend_transactions(&transactions);
    assert_eq!(gains_r, results_r);
    assert!(holding.inventory().is_empty());
    assert_eq!(holding.direction(), None);
}

#[test]
fn transactions_that_include_add_transfers() {
    let mut holding = Holding::default();
    let transactions = [
        Transaction::from("2020-04-01,Receive,100.0,25.0"),
        Transaction::from("2020-05-01,long,100.0,25.0"),
        Transaction::from("2020-06-01,Receive,100.0,25.0"),
        Transaction::from("2020-07-01,short,300.0,35.0"),
    ];
    let results_r = [
        Realized::from("2020-07-01,-100.0,3500.0,2020-04-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-07-01,-100.0,3500.0,2020-05-01,100.0,-2500.0,1000.0"),
        Realized::from("2020-07-01,-100.0,3500.0,2020-06-01,100.0,-2500.0,1000.0"),
    ];
    let gains_r = holding.extend_transactions(&transactions);
    assert_eq!(gains_r, results_r);
    assert!(holding.inventory().is_empty());
    assert_eq!(holding.direction(), None);
}

#[test]
fn transactions_that_include_remove_transfers_with_zero_gain() {
    let mut holding = Holding::default();
    let transactions = [
        Transaction::from("2020-04-01,Receive,100.0,25.0"),
        Transaction::from("2020-05-01,long,100.0,25.0"),
        Transaction::from("2020-06-01,Send,50.0,0.0"),
        Transaction::from("2020-07-01,short,150.0,35.0"),
    ];
    let results_r = [
//        Realized::from("2020-06-01,-50.0,1250.0,2020-04-01,50.0,-1250.0,0.0"),
        Realized::from("2020-07-01,-50.0,1750.0,2020-04-01,50.0,-1250.0,500.0"),
        Realized::from("2020-07-01,-100.0,3500.0,2020-05-01,100.0,-2500.0,1000.0"),
    ];
    let gains_r = holding.extend_transactions(&transactions);
    assert_eq!(gains_r, results_r);
    assert!(holding.inventory().is_empty());
    assert_eq!(holding.direction(), None);
}

#[test]
fn remove_inventory_with_zerogain_show_output() {
    let mut holding = Holding::default();
    holding.add_config("REALIZED_REMOVED_VALUE_AT_COST");
    let transactions = [
        Transaction::from("2020-03-01,long,100.0,20.0"),
        Transaction::from("2020-04-01,Receive,100.0,25.0"),
        Transaction::from("2020-05-01,long,100.0,30.0"),
        Transaction::from("2020-06-01,Send,50.0,35.0"),
        Transaction::from("2020-07-01,Send,100.0,35.0"),
        Transaction::from("2020-08-01,Send,50.0,35.0"),
    ];
    let results_ur = [URealized::from("2020-05-01,100.0,-3000.0")];

    let gains_r = holding.extend_transactions(&transactions[0..=2]);
    assert!(gains_r.is_empty());

    // partial send
    let gains_r = holding.add_transaction(&transactions[3]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-06-01,-50.0,1000.0,2020-03-01,50.0,-1000.0,0.0")));

    // larger send
    let gains_r = holding.add_transaction(&transactions[4]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-07-01,-50.0,1000.0,2020-03-01,50.0,-1000.0,0.0"),
        Realized::from("2020-07-01,-50.0,1250.0,2020-04-01,50.0,-1250.0,0.0"),
    ));

    // equal send
    let gains_r = holding.add_transaction(&transactions[5]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-08-01,-50.0,1250.0,2020-04-01,50.0,-1250.0,0.0")));

    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn remove_inventory_with_gain_at_market() {
    let mut holding = Holding::default();
    holding.add_config("REMOVED_VALUE_AT_MARKET");
    let transactions = [
        Transaction::from("2020-03-01,long,100.0,20.0"),
        Transaction::from("2020-04-01,Receive,100.0,25.0"),
        Transaction::from("2020-05-01,long,100.0,30.0"),
        Transaction::from("2020-06-01,Send,50.0,35.0"),
        Transaction::from("2020-07-01,Send,100.0,35.0"),
        Transaction::from("2020-08-01,Send,50.0,35.0"),
    ];
    let results_ur = [URealized::from("2020-05-01,100.0,-3000.0")];

    let gains_r = holding.extend_transactions(&transactions[0..=2]);
    assert!(gains_r.is_empty());

    // partial send
    let gains_r = holding.add_transaction(&transactions[3]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-06-01,-50.0,1750.0,2020-03-01,50.0,-1000.0,750.0")));

    // larger send
    let gains_r = holding.add_transaction(&transactions[4]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-07-01,-50.0,1750.0,2020-03-01,50.0,-1000.0,750.0"),
        Realized::from("2020-07-01,-50.0,1750.0,2020-04-01,50.0,-1250.0,500.0"),
    ));

    // equal send
    let gains_r = holding.add_transaction(&transactions[5]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-08-01,-50.0,1750.0,2020-04-01,50.0,-1250.0,500.0")));

    assert_eq!(holding.inventory(), results_ur);
}

#[test]
fn remove_inventory_with_gain_at_zero_value() {
    let mut holding = Holding::default();
    holding.add_config("REMOVED_VALUE_AT_ZERO");
    let transactions = [
        Transaction::from("2020-03-01,long,100.0,20.0"),
        Transaction::from("2020-04-01,Receive,100.0,25.0"),
        Transaction::from("2020-05-01,long,100.0,30.0"),
        Transaction::from("2020-06-01,Send,50.0,35.0"),
        Transaction::from("2020-07-01,Send,100.0,35.0"),
        Transaction::from("2020-08-01,Send,50.0,35.0"),
    ];
    let results_ur = [URealized::from("2020-05-01,100.0,-3000.0")];

    let gains_r = holding.extend_transactions(&transactions[0..=2]);
    assert!(gains_r.is_empty());

    // partial send
    let gains_r = holding.add_transaction(&transactions[3]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-06-01,-50.0,0.0,2020-03-01,50.0,-1000.0,-1000.0")));

    // larger send
    let gains_r = holding.add_transaction(&transactions[4]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-07-01,-50.0,0.0,2020-03-01,50.0,-1000.0,-1000.0"),
        Realized::from("2020-07-01,-50.0,0.0,2020-04-01,50.0,-1250.0,-1250.0"),
    ));

    // equal send
    let gains_r = holding.add_transaction(&transactions[5]);
    assert_eq!(gains_r, vec!(
        Realized::from("2020-08-01,-50.0,0.0,2020-04-01,50.0,-1250.0,-1250.0")));

    assert_eq!(holding.inventory(), results_ur);
}
