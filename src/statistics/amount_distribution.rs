use crate::order::Order;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum AmountBucket {
    Under50,
    From50To100,
    From100To500,
    Over500,
}

impl AmountBucket {
    fn from_amount(amount: f64) -> Self {
        if amount < 50.0 {
            AmountBucket::Under50
        } else if amount < 100.0 {
            AmountBucket::From50To100
        } else if amount < 500.0 {
            AmountBucket::From100To500
        } else {
            AmountBucket::Over500
        }
    }

    fn label(&self) -> &'static str {
        match self {
            AmountBucket::Under50 => "$0 - $50",
            AmountBucket::From50To100 => "$50 - $100",
            AmountBucket::From100To500 => "$100 - $500",
            AmountBucket::Over500 => "$500+",
        }
    }
}

pub struct AmountDistribution {
    counts: [usize; 4],
    total_orders: usize,
}

impl AmountDistribution {
    pub fn new() -> Self {
        Self {
            counts: [0; 4],
            total_orders: 0,
        }
    }

    fn bucket_index(bucket: AmountBucket) -> usize {
        match bucket {
            AmountBucket::Under50 => 0,
            AmountBucket::From50To100 => 1,
            AmountBucket::From100To500 => 2,
            AmountBucket::Over500 => 3,
        }
    }

    fn count(&self, bucket: AmountBucket) -> usize {
        self.counts[Self::bucket_index(bucket)]
    }

    fn percentage(&self, bucket: AmountBucket) -> f64 {
        if self.total_orders > 0 {
            (self.count(bucket) as f64 / self.total_orders as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl AmountDistribution {
    pub fn accept(&mut self, order: &Order) {
        let bucket = AmountBucket::from_amount(order.amount);
        self.counts[Self::bucket_index(bucket)] += 1;
        self.total_orders += 1;
    }

    pub fn merge(&mut self, other: AmountDistribution) {
        self.total_orders += other.total_orders;
        for i in 0..4 {
            self.counts[i] += other.counts[i];
        }
    }
}

impl Display for AmountDistribution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Amount Distribution ---")?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Range", "Count", "Percentage"]);

        for bucket in AmountBucket::iter() {
            table.add_row(vec![
                bucket.label().to_string(),
                self.count(bucket).to_string(),
                format!("{:.2} %", self.percentage(bucket)),
            ]);
        }

        writeln!(f, "{table}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderStatus;

    fn create_order(id: u32, customer: &str, amount: f64, status: OrderStatus) -> Order {
        Order {
            id,
            customer: customer.to_string(),
            amount,
            status,
        }
    }

    #[test]
    fn new_distribution_is_empty() {
        let dist = AmountDistribution::new();

        assert_eq!(dist.total_orders, 0);
        for bucket in AmountBucket::iter() {
            assert_eq!(dist.count(bucket), 0);
        }
    }

    #[test]
    fn bucket_from_amount_under_50() {
        assert_eq!(AmountBucket::from_amount(0.0), AmountBucket::Under50);
        assert_eq!(AmountBucket::from_amount(25.0), AmountBucket::Under50);
        assert_eq!(AmountBucket::from_amount(49.99), AmountBucket::Under50);
    }

    #[test]
    fn bucket_from_amount_50_to_100() {
        assert_eq!(AmountBucket::from_amount(50.0), AmountBucket::From50To100);
        assert_eq!(AmountBucket::from_amount(75.0), AmountBucket::From50To100);
        assert_eq!(AmountBucket::from_amount(99.99), AmountBucket::From50To100);
    }

    #[test]
    fn bucket_from_amount_100_to_500() {
        assert_eq!(AmountBucket::from_amount(100.0), AmountBucket::From100To500);
        assert_eq!(AmountBucket::from_amount(250.0), AmountBucket::From100To500);
        assert_eq!(AmountBucket::from_amount(499.99), AmountBucket::From100To500);
    }

    #[test]
    fn bucket_from_amount_over_500() {
        assert_eq!(AmountBucket::from_amount(500.0), AmountBucket::Over500);
        assert_eq!(AmountBucket::from_amount(1000.0), AmountBucket::Over500);
    }

    #[test]
    fn accept_increments_correct_bucket() {
        let mut dist = AmountDistribution::new();

        dist.accept(&create_order(1, "John", 25.0, OrderStatus::Paid));
        dist.accept(&create_order(2, "Jane", 75.0, OrderStatus::Paid));
        dist.accept(&create_order(3, "Bob", 200.0, OrderStatus::Paid));
        dist.accept(&create_order(4, "Alice", 600.0, OrderStatus::Paid));

        assert_eq!(dist.count(AmountBucket::Under50), 1);
        assert_eq!(dist.count(AmountBucket::From50To100), 1);
        assert_eq!(dist.count(AmountBucket::From100To500), 1);
        assert_eq!(dist.count(AmountBucket::Over500), 1);
        assert_eq!(dist.total_orders, 4);
    }

    #[test]
    fn percentage_calculates_correctly() {
        let mut dist = AmountDistribution::new();

        dist.accept(&create_order(1, "John", 25.0, OrderStatus::Paid));
        dist.accept(&create_order(2, "Jane", 25.0, OrderStatus::Paid));
        dist.accept(&create_order(3, "Bob", 75.0, OrderStatus::Paid));
        dist.accept(&create_order(4, "Alice", 600.0, OrderStatus::Paid));

        assert!((dist.percentage(AmountBucket::Under50) - 50.0).abs() < f64::EPSILON);
        assert!((dist.percentage(AmountBucket::From50To100) - 25.0).abs() < f64::EPSILON);
        assert!((dist.percentage(AmountBucket::From100To500) - 0.0).abs() < f64::EPSILON);
        assert!((dist.percentage(AmountBucket::Over500) - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentage_returns_zero_for_empty() {
        let dist = AmountDistribution::new();

        for bucket in AmountBucket::iter() {
            assert!((dist.percentage(bucket) - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn render_contains_header() {
        let dist = AmountDistribution::new();
        let output = dist.to_string();

        assert!(output.contains("--- Amount Distribution ---"));
    }

    #[test]
    fn merge_combines_counts() {
        let mut dist1 = AmountDistribution::new();
        dist1.accept(&create_order(1, "John", 25.0, OrderStatus::Paid));
        dist1.accept(&create_order(2, "Jane", 75.0, OrderStatus::Paid));

        let mut dist2 = AmountDistribution::new();
        dist2.accept(&create_order(3, "Bob", 25.0, OrderStatus::Paid));
        dist2.accept(&create_order(4, "Alice", 600.0, OrderStatus::Paid));

        dist1.merge(dist2);

        assert_eq!(dist1.total_orders, 4);
        assert_eq!(dist1.count(AmountBucket::Under50), 2);
        assert_eq!(dist1.count(AmountBucket::From50To100), 1);
        assert_eq!(dist1.count(AmountBucket::Over500), 1);
    }
}
