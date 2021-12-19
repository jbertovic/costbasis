//! CostBasis library maintains unrealized transactions in a holding.  As inventory change transactions
//! are added, the inventory is updated or removed into a realized gain.
//!
//! - `Holding` - maintains the current inventory and determines change impacts in inventory/unrealized
//!     and realized gains/loss
//!
//! - `URealized` - is a transaction record not realized (open position)
//! - `Realized` - is a combination of a transaction open and matching close, captures gain/loss
//! - `Transaction` - is a record of inventory change.  Can be replaced by a user defined struct that implements `Inventory` and `VolumeSplit` trait
//!
//! Assumes FIFO for maintaining holding inventory.  Future plans to add Lot, Avg Weight, and LIFO.
//!
//! Example
//! ```
//! use costbasis::holding::Holding;
//! use costbasis::inventory::InventoryType;
//! use costbasis::realized::Realized;
//! use costbasis::transaction::Transaction;
//! use costbasis::unrealized::URealized;
//!
//! let transactions = [
//!     Transaction::from("2020-01-01,long,200.0,25.0"),
//!     Transaction::from("2020-02-01,short,100.0,35.0"),
//! ];
//!
//! let mut holding = Holding::new(&transactions[0]);
//! let gains_realized = holding.add_transaction(&transactions[1]);
//!
//! // remaining inventory left in holding
//! let results_urealized = vec![URealized::from("2020-01-01,100.0,-2500.0")];
//!
//! // in the form of close date, quantity, proceeds, open date, cost
//! let results_realized = [Realized::from(
//!     "2020-02-01,-100.0,3500.0,2020-01-01,-2500.0",
//! )];
//!
//! assert_eq!(gains_realized, results_realized);
//! assert_eq!(holding.inventory(), results_urealized);
//! ```
//!
//! Look also in the examples directory.

const MARGIN_ERROR_QUANTITY: f64 = 0.0000000001;

/// holds struct and functions dealing with a `Holding`
pub mod holding;
/// traits to use with holding if user defined struct instead of using `Transaction`
pub mod inventory;
/// struct and functions related to `Realized` - realized gains/losses
pub mod realized;
/// defined `Transaction` struct to use in identifying inventory changes
pub mod transaction;
/// struct and functions related to `URealized` - unrealized inventory
pub mod unrealized;

#[cfg(test)]
mod tests {}
