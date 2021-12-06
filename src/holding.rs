use std::collections::HashSet;
use crate::inventory::{Inventory, InventoryType, VolumeSplit};
use crate::realized::Realized;
use crate::unrealized::URealized;
use crate::MARGIN_ERROR_QUANTITY;
use std::fmt;


/// Holding contains a set of `URealized` inventory.
/// 
/// add_transaction -> create matching size -> 
///     1) add to position and create unrealized OR 2) close position and create realized
/// 
/// Adding inventory such as a Deposit/Transfer of Stock or Receiving Coin must include a basis
/// and is treated the same as a Long Transaction
/// 
/// Removing inventory such as a Withdraw of stock or Sending Crypto has a few options
/// when treating if it creates gain or not
/// 
/// Default => removal of inventory is removed at current cost basis, no response
/// ADD_REALIZED_FOR_REMOVED => same as default but will respond with zero gain
/// 
/// All inventory is treated as FIFO
/// 
#[derive(Debug, Default)]
pub struct Holding {
    unrealized: Vec<URealized>,
    direction: Option<InventoryType>,
    config: HashSet<String>,
}

impl From<&[URealized]> for Holding {
    fn from(g: &[URealized]) -> Self {
        let mut gains = Holding::default();
        gains.unrealized.extend(g);
        // sum to determine direction
        let quantity: f64 = g.iter().map(|u| u.quantity()).sum();
        if quantity > 0.0 {
            gains.direction = Some(InventoryType::Long);
        } else {
            gains.direction = Some(InventoryType::Short);
        }
        gains
    }
}

impl Holding {
    pub fn new<T>(inv: &T) -> Self
    where
        T: Inventory + VolumeSplit<T> + Copy,
    {
        let mut gains = Holding::default();
        gains.add_transaction(inv);
        gains
    }

    // ASSUMES FIFO FOR NOW -> potential options for LIFO or LOTS or Avg Cost
    // need to check to see if its Inventory Add or Remove
    // vs a Buy/Sell Transaction
    // Buy is similar to Add but Remove can create different types of
    //     gains depending on Accounting rules; gifting, transaction fees (crypto)
    /// Transaction is an inventory change of Add/Deposit/Receive, Remove/Use/Send, Buy/Long, Short/Sell
    pub fn add_transaction<T>(&mut self, inv: &T) -> Vec<Realized>
    where
        T: Inventory + VolumeSplit<T> + Copy,
    { 
        // same direction of inventory change or empty inventory - add to inventory and exit with zero realized
        //if self.unrealized.is_empty() || self.direction == Some(inv.itype()) {
        if self.match_direction(inv) {
            self.add_inventory(inv.into());
            vec![]
        // change is in opposite direction which will remove inventory and create gains
        } else {
            // split_matches creating equal pairs between inv change and unrealized inventory
            let split_inv = self.split_matching_first(*inv);
            let mut realized_return = vec![];
            // create realized and remove matches as long as there is inventory
            if split_inv.len()>1 {
                realized_return.push(self.match_close(&split_inv[0]));
                realized_return.extend(self.add_transaction(&split_inv[1]));
            } else {
                realized_return.push(self.match_close(&split_inv[0]));
            }
            // Remove is handled differently depending on Configuration
            if inv.itype() == InventoryType::Remove {
                self.mod_removed(realized_return)
            } else {
                realized_return
            }
        }
    }

    pub fn extend_transactions<T>(&mut self, invs: &[T]) -> Vec<Realized>
    where
        T: Inventory + VolumeSplit<T> + Copy,
    {
        // add transactions one by one to keep any realized gains created
        let mut gains_r: Vec<Realized> = Vec::new();
        for inv in invs {
            gains_r.extend(self.add_transaction(inv));
        }
        gains_r
    }

    fn mod_removed(&self, mut realized: Vec<Realized>) -> Vec<Realized>
    {
        if self.config.contains("REALIZED_REMOVED_VALUE_AT_COST") {
            // shows removed realized at cost basis and zero gains
            realized.iter_mut().for_each(|r| r.zero_profit());
            realized           
        } else if self.config.contains("REMOVED_VALUE_AT_MARKET") {
            // assumes market price is in inventory change data as price or basis
            realized
        } else if self.config.contains("REMOVED_VALUE_AT_ZERO") {
            // Force proceeds at zero value
            realized.iter_mut().for_each(|r| r.zero_value());
            realized
        } else {
            vec!()
        }
    }


    pub fn add_inventory(&mut self, ur: URealized) {
        if self.direction.is_none() {
            self.direction = Some(ur.itype());
        }
        self.unrealized.push(ur);
    }

    /// remove inventory and return cost basis that is removed from inventory
    pub fn remove_inventory(&mut self, quantity: f64) -> f64 {
        let mut removed_basis = 0.0;
        let mut quantity_remaining = quantity;

        // start at the top of the que and remove until quantity is reached
        loop {
            if (self.unrealized[0].quantity() - quantity_remaining).abs() < MARGIN_ERROR_QUANTITY {
                removed_basis += self.unrealized[0].basis();
                self.unrealized.remove(0);
                break;
            } else if self.unrealized[0].quantity() < quantity_remaining {
                removed_basis += self.unrealized[0].basis();
                quantity_remaining -= self.unrealized[0].quantity();
                self.unrealized.remove(0);
            } else {
                let basis =
                    self.unrealized[0].basis() / self.unrealized[0].quantity() * quantity_remaining;
                removed_basis += basis;
                self.unrealized[0] = URealized::new(
                    self.unrealized[0].date(),
                    self.unrealized[0].quantity() - quantity_remaining,
                    self.unrealized[0].basis() - basis,
                );
                break;
            }
        }
        // if empty than reset direction
        self.check_zero_reset();
        removed_basis
    }

    fn match_close<T>(&mut self, inv: &T) -> Realized 
    where T: Inventory
    {   // can create panic if volumes don't match
        let unrealized = self.unrealized.remove(0);
        self.check_zero_reset();
        Realized::match_close(inv, &unrealized)
    }

    pub fn direction(&self) -> Option<InventoryType> {
        self.direction.clone()
    }

    fn match_direction<T>(&self, inv: &T) -> bool 
    where T: Inventory,
    {
        if self.direction.is_none() || self.direction == Some(inv.direction_type()) {
            true
        } else {
            false
        }
    }

    pub fn inventory(&self) -> Vec<URealized> {
        self.unrealized.clone()
    }

    pub fn position(&self) -> (f64, f64, f64) {
        // return quantity, price per unit, total basis
        let mut q = 0.0;
        let mut b = 0.0;
        let mut p = 0.0;

        for ur in self.unrealized.iter() {
            q += ur.quantity();
            b += ur.basis();
        }

        if q.abs() > MARGIN_ERROR_QUANTITY {
            p = (-b / q * 10000.0).round() / 10000.0;
        }
        (q, p, b)
    }

    fn check_zero_reset(&mut self) {
        if self.unrealized.is_empty() || self.position().0.abs() < MARGIN_ERROR_QUANTITY {
            self.direction = None;
            self.unrealized = vec!();
        }
    }

    // builds first match between inv changes and inventory unrealized 
    // quantity has to be in opposite directions between inv and unrealized
    // remaining inv changes returned or inventory modified to match
    fn split_matching_first<T>(&mut self, inv: T) -> Vec<T>
    where T: Inventory + VolumeSplit<T> + Clone
    {
        if (inv.quantity() + self.unrealized[0].quantity()).abs() < MARGIN_ERROR_QUANTITY {
            vec!(inv)
        } else if self.unrealized[0].quantity().abs() > inv.quantity().abs() {
            // split first inventory into two
            let (close_ur, modified_inv) = self.unrealized[0].split(inv.quantity().abs());
            self.unrealized.remove(0);
            self.unrealized.insert(0, modified_inv);
            self.unrealized.insert(0, close_ur);
            vec!(inv)
        } else {
            let (match_trans, remaining_trans) = inv.split(self.unrealized[0].quantity().abs());
            vec!(match_trans, remaining_trans)
        }
    }

    pub fn add_config(&mut self, c: &str) {
        self.config.insert(c.to_owned());
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Position; quantity:{:.4}, price:{:.4}, basis:{:.4}, inventory_count:{}", 
            self.position().0, self.position().1, self.position().2, self.unrealized.len())
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_new_gains_from_unrealized_slice() {
        let results_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,100.0,-2500.0"),
            URealized::from("2020-03-01,100.0,-2500.0"),
        ];
        let holding = Holding::from(&results_ur[..]);
        assert_eq!(holding.inventory(), results_ur);
        assert_eq!(holding.direction(), Some(InventoryType::Long));
    }

    #[test]
    fn modify_inventory_with_transfer_of_basis() {
        let starting_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,100.0,-2500.0"),
        ];
        let mut holding = Holding::from(&starting_ur[..]);
        let results_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,100.0,-2500.0"),
            URealized::from("2020-02-01,100.0,-3000.0"),
            URealized::from("2020-03-01,100.0,-2500.0"),
        ];

        // add inventory
        holding.add_inventory(URealized::from("2020-02-01,100.0,-3000.0"));
        holding.add_inventory(URealized::from("2020-03-01,100.0,-2500.0"));

        assert_eq!(holding.inventory(), results_ur);
    }

    #[test]
    fn remove_inventory_with_transfer_of_basis() {
        let starting_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,200.0,-6000.0"),
            URealized::from("2020-03-01,300.0,-9000.0"),
        ];
        let mut holding = Holding::from(&starting_ur[..]);
        let results_ur = [URealized::from("2020-03-01,300.0,-9000.0")];

        // partial remove
        let removed_basis = holding.remove_inventory(50.0);
        assert_eq!(removed_basis, -1250.0);

        // larger remove
        let removed_basis = holding.remove_inventory(150.0);
        assert_eq!(removed_basis, -4250.0);

        // equal remove
        let removed_basis = holding.remove_inventory(100.0);
        assert_eq!(removed_basis, -3000.0);

        assert_eq!(holding.inventory(), results_ur);
    }

    #[test]
    fn remove_all_inventory() {
        let starting_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,200.0,-6000.0"),
        ];
        let mut holding = Holding::from(&starting_ur[..]);
        let removed_basis = holding.remove_inventory(300.0);
        assert_eq!(removed_basis, -8500.0);
        assert!(holding.inventory().is_empty());
        assert_eq!(holding.direction(), None);
    }

    #[test]
    fn working_small_quantities_removed() {
        let starting_ur = [
            URealized::from("2020-01-01,0.000000433,-0.032475"),
        ];
        let mut holding = Holding::from(&starting_ur[..]);
        let removed_basis = holding.remove_inventory(0.00000043301);
        assert_eq!(removed_basis, -0.032475);
        assert!(holding.inventory().is_empty());
        assert_eq!(holding.direction(), None);
    }

    #[test]
    fn position_summary() {
        let starting_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,200.0,-6000.0"),
            URealized::from("2020-03-01,300.0,-9000.0"),
        ];
        let holding = Holding::from(&starting_ur[..]);

        assert_eq!(holding.position(), (600.0, 29.1667, -17500.0));
    }

    #[test]
    fn if_holding_is_empty_position_returns_zeros() {
        let holding = Holding::default();
        assert_eq!(holding.position(), (0.0, 0.0, 0.0));
    }

    #[test]
    fn split_match_first_with_variety_of_changes() {
        let starting_ur = [
            URealized::from("2020-01-01,100.0,-2500.0"),
            URealized::from("2020-02-01,200.0,-5000.0"),
            URealized::from("2020-03-01,300.0,-7500.0"),
        ];
        
        // tests equal inv_change
        let mut holding = Holding::from(&starting_ur[..]);
        let matches = holding.split_matching_first(URealized::from("2020-04-01,-100.0,3000.0"));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], URealized::from("2020-04-01,-100.0,3000.0"));
        assert_eq!(holding.unrealized, starting_ur);
        
        // tests smaller inv_change
        let mut holding = Holding::from(&starting_ur[..]);
        let matches = holding.split_matching_first(URealized::from("2020-04-01,-50.0,1500.0"));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], URealized::from("2020-04-01,-50.0,1500.0"));
        assert_eq!(holding.unrealized[0], URealized::from("2020-01-01,50.0,-1250.0"));
        assert_eq!(holding.unrealized[1], URealized::from("2020-01-01,50.0,-1250.0"));
        assert_eq!(holding.unrealized[2..3], starting_ur[1..2]);

        // tests larger inv_change
        let mut holding = Holding::from(&starting_ur[..]);
        let matches = holding.split_matching_first(URealized::from("2020-04-01,-350.0,10500.0"));
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], URealized::from("2020-04-01,-100.0,3000.0"));
        assert_eq!(matches[1], URealized::from("2020-04-01,-250.0,7500.0"));
        assert_eq!(holding.unrealized[0], URealized::from("2020-01-01,100.0,-2500.0"));
        assert_eq!(holding.unrealized[1], URealized::from("2020-02-01,200.0,-5000.0"));
        assert_eq!(holding.unrealized[2], URealized::from("2020-03-01,300.0,-7500.0"));
    }

}
