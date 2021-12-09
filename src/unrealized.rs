use crate::inventory::{Inventory, InventoryType, VolumeSplit};
use chrono::NaiveDate;
use std::fmt;

/// open date, open quantity, open value
/// quantity is positive for long and negative for short, value is full basis not just price
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct URealized(NaiveDate, f64, f64);

impl From<&str> for URealized {
    fn from(s: &str) -> Self {
        let field: Vec<&str> = s.split(',').collect();
        URealized {
            0: NaiveDate::parse_from_str(field[0], "%Y-%m-%d").unwrap(),
            1: field[1].parse().unwrap(),
            2: field[2].parse().unwrap(),
        }
    }
}

impl URealized {
    pub fn new(date: NaiveDate, quantity: f64, basis: f64) -> Self {
        URealized {
            0: date,
            1: quantity,
            2: basis,
        }
    }
}

impl fmt::Display for URealized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "URealized: {}, quantity: {:.4}, price: {:.4}, basis: {:.4}", 
            self.0, self.1, -self.2/self.1, self.2)
    }
}

impl VolumeSplit<URealized> for URealized {
    /// first return is the closed portion and 2nd return is left over inventory
    fn split(&self, mut quantity: f64) -> (URealized, URealized) {
        if self.1 < 0.0 {
            quantity *= -1.0;
        }
        let split1 = URealized {
            0: self.0,
            1: quantity,
            2: self.2 * quantity / self.1,
        };
        let split2 = URealized {
            0: self.0,
            1: self.1 - quantity,
            2: self.2 * (self.1 - quantity) / self.1,
        };
        (split1, split2)
    }
}

impl<T> From<&T> for URealized
where
    T: Inventory,
{
    fn from(inv: &T) -> Self {
        Self {
            0: inv.date(),
            1: inv.quantity(),
            2: inv.basis(),
        }
    }
}

impl Inventory for URealized {
    fn date(&self) -> NaiveDate {
        self.0
    }

    fn quantity(&self) -> f64 {
        self.1
    }

    fn basis(&self) -> f64 {
        self.2
    }

    fn itype(&self) -> InventoryType {
        self.direction_type()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn split_gain_unrealized_into_two_long() {
        let start_ur = URealized::from("2020-01-01,200.0,-5000.0");
        let result_close = URealized::from("2020-01-01,50.0,-1250.0");
        let result_inv_remaining = URealized::from("2020-01-01,150.0,-3750.0");
        let (split1, split2) = start_ur.split(50.0);
        assert_eq!(split1, result_close);
        assert_eq!(split2, result_inv_remaining);
    }

    #[test]
    fn split_gain_unrealized_into_two_short() {
        let start_ur = URealized::from("2020-01-01,-200.0,5000.0");
        let result_close = URealized::from("2020-01-01,-50.0,1250.0");
        let result_inv_remaining = URealized::from("2020-01-01,-150.0,3750.0");
        let (split1, split2) = start_ur.split(50.0);
        assert_eq!(split1, result_close);
        assert_eq!(split2, result_inv_remaining);
    }


}
