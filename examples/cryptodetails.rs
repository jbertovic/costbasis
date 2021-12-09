/// Example taking crypto buys and sells to determine realized gains and remaining inventory
/// Example uses data from my coinbase acct and ethereum wallet

use costbasis::realized::{Realized, total_realized, realized_to_compact};
use costbasis::holding::Holding;
use costbasis::transaction::Transaction;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;


fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("./examples/CRYPTO_TRANS.csv")?;
    let symbol = "ETH";

    // load csv file in format: timestamp, ttype, symbol, asset, price
    // convert into hashmap of transactions, symbol is key and value is vector of transactions
    let symbol_transaction_data = get_transactions(f)?;
    println!("TRANSACTIONS LOADED");

    let mut symbol_holding_realized: HashMap<String, (Holding, Vec<Realized>)> = HashMap::new();
    for (symbol, transactions) in symbol_transaction_data.iter() {
        let mut holding = Holding::default();
        //holding.add_config("REALIZED_REMOVED_VALUE_AT_COST");
        let realized = holding.extend_transactions(transactions);
        symbol_holding_realized.insert(symbol.to_owned(),(holding, realized));
    }
    println!("HOLDINGS CALCULATED");

    println!("-------------------------------------------------------------");
    //for (symbol, (holding, realized)) in symbol_holding_realized.iter() {
    let (holding, realized) = symbol_holding_realized.get(symbol).unwrap();
    println!("SYMBOL: {} __ {}", symbol, holding);
    println!("REALIZED RETURNS: {:.2}", total_realized(&realized));
    if !realized.is_empty() {
        for r in realized_to_compact(&realized).iter() {
            println!("{}", r);
        }
    }
    println!("-------------------------------------------------------------");
    println!("DETAILED RETURNS: ");
    realized.iter().for_each(|r| println!("{}", r));
    println!("-------------------------------------------------------------");
    println!("INVENTORY: ");
    holding.inventory().iter().for_each(|ur| println!("{}", ur));

    Ok(())
}


fn get_transactions(f: File) -> Result<HashMap<String, Vec<Transaction>>, Box<dyn Error>> {
    let mut transrec: HashMap<String, Vec<Transaction>> = HashMap::new();

    let lines = io::BufReader::new(f).lines();

    for l in lines.skip(1) {
        let line_data = l.unwrap();
        let line_data: Vec<&str> = line_data.split(",").collect();
        let trans_str = [line_data[0], line_data[1], line_data[3], line_data[4]].join(",");
        if let Some(trans) = transrec.get_mut(line_data[2]) {
            trans.push(Transaction::from(trans_str.as_str()));
        } else {
            transrec.insert(
                line_data[2].to_owned(),
                vec![Transaction::from(trans_str.as_str())],
            );
        }
    }
    Ok(transrec)
}
