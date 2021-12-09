use super::inventory::{Inventory, InventoryType, VolumeSplit};
use chrono::NaiveDate;
use std::str::FromStr;

impl std::str::FromStr for InventoryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Long" | "long" | "buy" | "BUY" | "b" | "l" => Ok(InventoryType::Long),
            "Short" | "short" | "sell" | "SELL" | "s" => Ok(InventoryType::Short),
            "Receive" | "Transfer_In" | "RECEIVE" | "Add" | "ADD" => Ok(InventoryType::Add),
            "Send" | "Transfer_Out" | "SEND" | "Remove" | "REMOVE" => Ok(InventoryType::Remove),
            _ => Err(format!("'{}' is not a valid value for InventoryType", s)),
        }
    }
}

/// Transaction
/// transaction date, transaction type, quantity, price
/// 
/// Implements Inventory and VolumeSplit traits to be used in Holdings Struct.
/// 
/// User can implement their own Transaction Struct by implementing both Inventory and VolumeSplit
/// 
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transaction(NaiveDate, InventoryType, f64, f64);

impl From<&str> for Transaction {
    fn from(s: &str) -> Self {
        let field: Vec<&str> = s.split(',').collect();
        Transaction {
            0: NaiveDate::parse_from_str(field[0], "%Y-%m-%d").unwrap(),
            1: InventoryType::from_str(field[1]).unwrap(),
            2: field[2].parse().unwrap(),
            3: field[3].parse().unwrap(),
        }
    }
}

impl Transaction {
    pub fn quant_multiplier(&self) -> f64 {
        match self.1 {
            InventoryType::Long | InventoryType::Add => 1.0,
            InventoryType::Short | InventoryType::Remove => -1.0,
        }
    }
}

impl Inventory for Transaction {
    fn basis(&self) -> f64 {
        self.2 * self.3 * self.quant_multiplier() * -1.0
    }

    fn quantity(&self) -> f64 {
        self.2 * self.quant_multiplier()
    }

    fn date(&self) -> NaiveDate {
        self.0
    }

    fn itype(&self) -> InventoryType {
        self.1.clone()
    }
}

impl VolumeSplit<Transaction> for Transaction {
    fn split(&self, quantity: f64) -> (Transaction, Transaction) {
        let split1 = Transaction {
            0: self.0,
            1: self.1.clone(),
            2: quantity,
            3: self.3,
        };
        let split2 = Transaction {
            0: self.0,
            1: self.1.clone(),
            2: self.2 - quantity,
            3: self.3,
        };
        (split1, split2)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_split_for_transaction_long() {
        let trans = Transaction::from("2020-01-01,long,100.0,25.0");
        let (trans1, trans2) = trans.split(25.0);

        let result1 = Transaction::from("2020-01-01,long,25.0,25.0");
        let result2 = Transaction::from("2020-01-01,long,75.0,25.0");

        assert_eq!(trans1, result1);
        assert_eq!(trans2, result2);
    }

    #[test]
    fn test_split_for_transaction_short() {
        let trans = Transaction::from("2020-01-01,short,100.0,25.0");
        let (trans1, trans2) = trans.split(25.0);

        let result1 = Transaction::from("2020-01-01,short,25.0,25.0");
        let result2 = Transaction::from("2020-01-01,short,75.0,25.0");

        assert_eq!(trans1, result1);
        assert_eq!(trans2, result2);
    }


    #[test]
    fn test_transaction_long() {
        let trans = Transaction::from("2020-01-01,long,100.0,25.0");
        assert_eq!(trans.quant_multiplier(), 1.0);
        assert!(trans.quantity() > 0.0);
        assert_eq!(trans.basis(), -2500.0);
        assert_eq!(trans.date(), NaiveDate::from_ymd(2020, 1, 1));
        assert_eq!(trans.itype(), InventoryType::Long);
    }

    #[test]
    fn test_transaction_short() {
        let trans = Transaction::from("2020-02-01,short,200.0,10.0");
        assert_eq!(trans.quant_multiplier(), -1.0);
        assert!(trans.quantity() < 0.0);
        assert_eq!(trans.basis(), 2000.0);
        assert_eq!(trans.date(), NaiveDate::from_ymd(2020, 2, 1));
        assert_eq!(trans.itype(), InventoryType::Short);
    }
}
