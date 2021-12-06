use std::hash::Hash;
use std::collections::HashMap;
use crate::inventory::Inventory;
use crate::unrealized::URealized;
use chrono::NaiveDate;
use std::fmt;

/// close date, close quantity, close value, open date, open quantity, open value, realized gain
#[derive(Debug, PartialEq)]
pub struct Realized(NaiveDate, f64, f64, NaiveDate, f64, f64, f64);

impl Realized {
    pub fn match_close<T>(inv: &T, inv_ur: &URealized) -> Realized
    where
        T: Inventory,
    {
        // todo: include panic if volumes don't match and are in opposite directions
        Realized {
            0: inv.date(),
            1: inv.quantity(),
            2: inv.basis(),
            3: inv_ur.date(),
            4: inv_ur.quantity(),
            5: inv_ur.basis(),
            6: inv_ur.basis() + inv.basis(),
        }
    }

    pub fn zero_profit(&mut self) {
        self.2 = -self.5;
        self.6 = 0.0;
    }

    pub fn zero_value(&mut self) {
        self.2 = 0.0;
        self.6 = self.5;
    }
}

impl From<&str> for Realized {
    fn from(s: &str) -> Self {
        let field: Vec<&str> = s.split(',').collect();
        Realized {
            0: NaiveDate::parse_from_str(field[0], "%Y-%m-%d").unwrap(),
            1: field[1].parse().unwrap(),
            2: field[2].parse().unwrap(),
            3: NaiveDate::parse_from_str(field[3], "%Y-%m-%d").unwrap(),
            4: field[4].parse().unwrap(),
            5: field[5].parse().unwrap(),
            6: field[6].parse().unwrap(),
        }
    }
}

/// sales date, quantity, proceeds, costs, pl
#[derive(Debug, PartialEq)]
pub struct RealizedCompact(NaiveDate, f64, f64, String, f64, f64);

impl RealizedCompact {
    fn new(date: NaiveDate, quantity: f64, proceeds: f64, open_dates: String, costs: f64) -> Self {
        Self {
            0: date,
            1: quantity,
            2: proceeds,
            3: open_dates,
            4: costs,
            5: proceeds+costs,
        }
    }
}

impl fmt::Display for RealizedCompact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "close_date: {} quantity:{:.4}, proceeds:{:.2}, cost_basis:{:.2}, gain_loss:{:.2}", 
            self.0, self.1, self.2, self.4, self.5)
    }
}

impl From<&[Realized]> for RealizedCompact {
    fn from(realized: &[Realized]) -> Self {
        // need to check that all dates are the same?
        let date = realized[0].0;
        let quantity = realized.iter().map(|r| r.1).sum::<f64>().abs();
        let proceeds = realized.iter().map(|r| r.2).sum();
        // add a string of dates, or insert various
        let costs = realized.iter().map(|r| r.5).sum();
        Self::new(date, quantity, proceeds, String::from(""), costs)
    }
}

pub fn realized_to_compact(realized: &[Realized]) -> Vec<RealizedCompact> {
    // group by date - assumes slice is ordered
    // strip out column of dates to group by
    let dates: Vec<NaiveDate> = realized.iter().map(|r| r.0).collect();
    let group_index = group_by_index(&dates);
    group_index.iter().map(|i| RealizedCompact::from(&realized[i.0..i.1])).collect()
}

//https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=304098b59de431c9d77a537a88d8f269
fn group_by_index<T>(s: &[T]) -> Vec<(usize,usize)> 
where T: Eq + Hash + Copy
{
    let mut index: HashMap<T,usize> = HashMap::new();
    for (i, g) in s.iter().enumerate() {
        if !index.contains_key(g) {
            index.insert(*g, i);
        }
    }
    let mut slices_index: Vec<usize> = index.into_values().collect();
    slices_index.sort_unstable();
    
    // combine into start and end of each slice
    let mut slices_index_group: Vec<_> = slices_index.iter().copied().zip(slices_index[1..].iter().copied()).collect();
    slices_index_group.push((*slices_index.last().unwrap(), s.len()));
    slices_index_group  
}

pub fn total_realized(r: &[Realized]) -> f64 {
    r.iter().map(|r|r.6).sum()
}


#[cfg(test)]
mod tests {

    use super::*;

    fn set_realized() -> [Realized;5] {
        [
            Realized::from("2020-04-01,-100.0,3500.0,2020-01-01,100.0,-2500.0,1000.0"),
            Realized::from("2020-04-01,-100.0,3500.0,2020-02-01,100.0,-2500.0,1000.0"),
            Realized::from("2020-06-01,-100.0,3500.0,2020-05-01,100.0,-2500.0,1000.0"),
            Realized::from("2020-07-01,-100.0,3500.0,2020-05-01,100.0,-2500.0,1000.0"),
            Realized::from("2020-07-01,-100.0,3500.0,2020-05-15,100.0,-2500.0,1000.0"),
        ]
    }

    #[test]
    fn given_array_of_realized_group_by_close_date() {
        let result = [
            RealizedCompact::new(NaiveDate::from_ymd(2020,4,1), 200.0, 7000.0, String::new(), -5000.0),
            RealizedCompact::new(NaiveDate::from_ymd(2020,6,1), 100.0, 3500.0, String::new(), -2500.0),
            RealizedCompact::new(NaiveDate::from_ymd(2020,7,1), 200.0, 7000.0, String::new(), -5000.0),
        ];
        assert_eq!(realized_to_compact(&set_realized()), result);
    }

    #[test]
    fn given_array_of_realized_calculate_total_gain() {
        assert_eq!(total_realized(&set_realized()), 5000.0);
    }

}

