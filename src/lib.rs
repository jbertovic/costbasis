/// costbasis library maintains unrealized transactions in a holding and as transactions are added
/// realized gains are calculated and removed from the holding.
/// 
/// Holding - maintains the current inventory and determines change impacts in inventory/unrealized 
///     and realized gains/loss
/// 
/// Unrealized - is a transaction record not realized (open position)
/// Realized - is a combination of a transaction open and matching close, captures gain/loss
/// Transaction - is a record of inventory change

// Assumes FIFO for maintaining holding inventory

const MARGIN_ERROR_QUANTITY: f64 = 0.0000000001;

pub mod holding;
pub mod inventory;
pub mod realized;
pub mod transaction;
pub mod unrealized;

#[cfg(test)]
mod tests {}
