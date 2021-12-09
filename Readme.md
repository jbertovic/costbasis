I created this library to help track costbasis of my trading of equity, options and crypto.  I use this to help create a profit loss report.  I will continue developing this further to allow me to calculate capital gains for taxes similar to
what gainskeeper does.

CostBasis library maintains unrealized transactions in a holding. As inventory change transactions are added, the inventory is updated or removed into a realized gain.

- `Holding` - maintains the current inventory and determines change impacts in inventory/unrealized and realized gains/loss
- `URealized` - is a transaction record not realized (open position)
- `Realized` - is a combination of a transaction open and matching close, captures gain/loss

Transaction - is a record of inventory change. Can be replaced by a user defined struct that implements `Inventory` and `VolumeSplit` trait

Assumes FIFO for maintaining holding inventory. Future plans to add Lot, Avg Weight, and LIFO.

Look at examples `cryptogains` and `cryptodetails`.  This uses a csv of my Crypto transactions which is culmination of wallet, coinbase, and coinbase pro transactions.  I took csv reports from each of them and data wrangled it down to transaction inventory changes.  All transaction fees in ETH on the ethereum network are indicated as REMOVE in the data.  Internal Sends and Receives are not included as they aren't taxable events or inventory changes.

Also, you can look at some of the tests under `./tests/costbasis.rs`

A quick example:
 ```
 use costbasis::holding::Holding;
 use costbasis::inventory::InventoryType;
 use costbasis::realized::Realized;
 use costbasis::transaction::Transaction;
 use costbasis::unrealized::URealized;
 
 let transactions = [
     Transaction::from("2020-01-01,long,200.0,25.0"),
     Transaction::from("2020-02-01,short,100.0,35.0"),
 ];

 let mut holding = Holding::new(&transactions[0]);
 let gains_realized = holding.add_transaction(&transactions[1]);

 // remaining inventory left in holding
 let results_urealized = vec![URealized::from("2020-01-01,100.0,-2500.0")];
 
 // in the form of close date, close quantity, proceeds, open date, open quantity, cost, profit
 let results_realized = [Realized::from(
     "2020-02-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0",
 )];
 
 assert_eq!(gains_realized, results_realized);
 assert_eq!(holding.inventory(), results_urealized);
 ```

FUTURE:
- need to add better documentation
- add Avg Cost method - would work with mutual funds
- add LIFO method - alternative to the current FIFO implementation
- Lots method - not sure if this one is worth implementing
- `Realized` better access to members
- `Realized` identification of long-term vs short-term gains
- `Realized` a way to add adjustments like wash sales and dealing with basis transfer from options relative to the underlying
