use crate::inventory::Inventory;
use crate::unrealized::URealized;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;


// TODO: shrink down to one volume for Realized

/// Holds a realized match of open inventory change and close inventory change along with the
/// realized gain or loss.
///
/// For now use Display trait to view
// close date, quantity, close value, open date, open value, realized gain
#[derive(Debug, PartialEq)]
pub struct Realized(NaiveDate, f64, f64, NaiveDate, f64, f64);

impl Realized {

    pub fn new(c_date: NaiveDate, quantity: f64, c_basis: f64, o_date: NaiveDate, o_basis: f64) -> Self {
        Realized {
            0: c_date,
            1: quantity,
            2: c_basis,
            3: o_date,
            4: o_basis,
            5: c_basis + o_basis,
        }
    }

    pub fn match_close<T>(inv: &T, inv_ur: &URealized) -> Realized
    where
        T: Inventory,
    {
        // todo: include panic if volumes don't match and are in opposite directions
        Realized::new(inv.date(), inv.quantity(), inv.basis(), inv_ur.date(), inv_ur.basis())

        // Realized {
        //     0: inv.date(),
        //     1: inv.quantity(),
        //     2: inv.basis(),
        //     3: inv_ur.date(),
        //     4: inv_ur.quantity(),
        //     5: inv_ur.basis(),
        //     6: inv_ur.basis() + inv.basis(),
        // }
    }

    // make crate private - only holding uses this function
    pub fn zero_profit(&mut self) {
        self.2 = -self.4;
        self.5 = 0.0;
    }
    // make crate private - only holding uses this function
    pub fn zero_value(&mut self) {
        self.2 = 0.0;
        self.5 = self.4;
    }

    // getters
    pub fn close_date(&self) -> NaiveDate {
        self.0
    }
    pub fn quantity(&self) -> f64 {
        self.1
    }
    pub fn close_basis(&self) -> f64 {
        self.2
    }
    pub fn open_date(&self) -> NaiveDate {
        self.3
    }
    pub fn open_basis(&self) -> f64 {
        self.4
    }
    pub fn realized(&self) -> f64 {
        self.5
    }

}

impl From<&str> for Realized {
    fn from(s: &str) -> Self {
        let field: Vec<&str> = s.split(',').collect();
        Realized::new(
            NaiveDate::parse_from_str(field[0], "%Y-%m-%d").unwrap(),
            field[1].parse().unwrap(),
            field[2].parse().unwrap(),
            NaiveDate::parse_from_str(field[3], "%Y-%m-%d").unwrap(),
            field[4].parse().unwrap(),
        )
    }
}

impl fmt::Display for Realized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "close_date: {} quantity:{:.4}, proceeds:{:.2}, open_date: {}, cost_basis:{:.2}, gain_loss:{:.2}", 
            self.0, self.1, self.2, self.3, self.4, self.5)
    }
}

/// A compact form of realized that may have multiple open dates.  Groups all open basis transactions based on the
/// close date.  This helps when you have for example one sale which covers multiple buys on a variety of dates.
// sales date, quantity, proceeds, costs, pl
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
            5: proceeds + costs,
        }
    }
}

impl fmt::Display for RealizedCompact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "close_date: {} quantity:{:.4}, proceeds:{:.2}, cost_basis:{:.2}, gain_loss:{:.2}",
            self.0, self.1, self.2, self.4, self.5
        )
    }
}

impl From<&[Realized]> for RealizedCompact {
    fn from(realized: &[Realized]) -> Self {
        // need to check that all dates are the same?
        let date = realized[0].0;
        let quantity = realized.iter().map(|r| r.1).sum::<f64>().abs();
        let proceeds = realized.iter().map(|r| r.2).sum();
        // add a string of dates, or insert various
        let costs = realized.iter().map(|r| r.4).sum();
        //TODO - string with multiple dates?
        Self::new(date, quantity, proceeds, String::from(""), costs)
    }
}

/// Convert slice of `Realized` into compact form by grouping by close date
pub fn realized_to_compact(realized: &[Realized]) -> Vec<RealizedCompact> {
    // group by date - assumes slice is ordered
    // strip out column of dates to group by
    let dates: Vec<NaiveDate> = realized.iter().map(|r| r.0).collect();
    let group_index = group_by_index(&dates);
    group_index
        .iter()
        .map(|i| RealizedCompact::from(&realized[i.0..i.1]))
        .collect()
}

//https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=304098b59de431c9d77a537a88d8f269
fn group_by_index<T>(s: &[T]) -> Vec<(usize, usize)>
where
    T: Eq + Hash + Copy,
{
    let mut index: HashMap<T, usize> = HashMap::new();
    for (i, g) in s.iter().enumerate() {
        if !index.contains_key(g) {
            index.insert(*g, i);
        }
    }
    let mut slices_index: Vec<usize> = index.into_values().collect();
    slices_index.sort_unstable();

    // combine into start and end of each slice
    let mut slices_index_group: Vec<_> = slices_index
        .iter()
        .copied()
        .zip(slices_index[1..].iter().copied())
        .collect();
    slices_index_group.push((*slices_index.last().unwrap(), s.len()));
    slices_index_group
}

/// Total realized is the sum of all profit / loss in the slice of `Realized`
pub fn total_realized(r: &[Realized]) -> f64 {
    r.iter().map(|r| r.5).sum()
}

#[cfg(test)]
mod tests {

    use super::*;

    fn set_realized() -> [Realized; 5] {
        [
            Realized::from("2020-04-01,-100.0,3500.0,2020-01-01,-2500.0"),
            Realized::from("2020-04-01,-100.0,3500.0,2020-02-01,-2500.0"),
            Realized::from("2020-06-01,-100.0,3500.0,2020-05-01,-2500.0"),
            Realized::from("2020-07-01,-100.0,3500.0,2020-05-01,-2500.0"),
            Realized::from("2020-07-01,-100.0,3500.0,2020-05-15,-2500.0"),
        ]
    }

    #[test]
    fn given_array_of_realized_group_by_close_date() {
        let result = [
            RealizedCompact::new(
                NaiveDate::from_ymd(2020, 4, 1),
                200.0,
                7000.0,
                String::new(),
                -5000.0,
            ),
            RealizedCompact::new(
                NaiveDate::from_ymd(2020, 6, 1),
                100.0,
                3500.0,
                String::new(),
                -2500.0,
            ),
            RealizedCompact::new(
                NaiveDate::from_ymd(2020, 7, 1),
                200.0,
                7000.0,
                String::new(),
                -5000.0,
            ),
        ];
        assert_eq!(realized_to_compact(&set_realized()), result);
    }

    #[test]
    fn given_array_of_realized_calculate_total_gain() {
        assert_eq!(total_realized(&set_realized()), 5000.0);
    }
}
