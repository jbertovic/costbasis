use chrono::NaiveDate;

pub trait Inventory {
    fn basis(&self) -> f64;

    fn quantity(&self) -> f64;

    fn date(&self) -> NaiveDate;

    fn itype(&self) -> InventoryType; 

    fn direction_type(&self) -> InventoryType {
        if self.quantity() > 0.0 {
            InventoryType::Long
        } else {
            InventoryType::Short
        }
    }
}

/// VolumeSplit is to divide into two parts.  Quantity is always positive.
pub trait VolumeSplit<T> {
    fn split(&self, quantity: f64) -> (T, T);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InventoryType {
    Long,
    Short,
    Add,
    Remove,
}

