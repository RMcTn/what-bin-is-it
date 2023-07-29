#![allow(clippy::needless_return)]
mod crawler;

use chrono::{Datelike, NaiveDate};
use std::dbg;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bins = crawler::get_stuff().await?;

    let today = chrono::Utc::now().date_naive();
    let next_collection_date = next_collection_date_from(today);

    dbg!(next_collection_date);
    let mut next_collection_day_for_bins = Vec::new();
    for bin in bins {
        let next_day = next_collection_date_for_bin(&bin, next_collection_date);
        next_collection_day_for_bins.push(next_day);
    }

    let mut closest_bin_day = next_collection_day_for_bins[0];
    let mut closest_bin_days = Vec::new();
    for bin_day in next_collection_day_for_bins {
        if bin_day.date == closest_bin_day.date {
            closest_bin_days.push(bin_day);
        }
        if bin_day.date < closest_bin_day.date {
            closest_bin_day = bin_day;
            closest_bin_days.clear();
            closest_bin_days.push(bin_day);
        }
    }

    let next_bin_collection = NextBinCollection {
        bins: closest_bin_days,
    };

    println!("Next bins:");
    for bins in next_bin_collection.bins {
        println!("{} bin is being collected on {}", bins.bin, bins.date);
    }

    return Ok(());
}

#[derive(Debug)]
struct TimeFromTarget {
    _target_date: NaiveDate,
    date: NaiveDate,
    how_far_from_target: chrono::Duration,
}
/// Filters out negatives (i.e dates from before the target_date)
fn calculate_differences_from_date(
    dates: &[NaiveDate],
    target_date: NaiveDate,
) -> Vec<TimeFromTarget> {
    let differences: Vec<TimeFromTarget> = dates
        .iter()
        .map(|date| TimeFromTarget {
            _target_date: target_date,
            date: *date,
            how_far_from_target: *date - target_date,
        })
        .filter(|time_from_target| time_from_target.how_far_from_target.num_seconds() >= 0)
        .collect();
    return differences;
}

fn next_collection_date_for_bin(
    bin_dates: &BinDates,
    target_date: NaiveDate,
) -> NextBinCollectionDay {
    let diffs = calculate_differences_from_date(&bin_dates.dates, target_date);

    let closest_day = diffs
        .iter()
        .min_by(|x, y| x.how_far_from_target.cmp(&y.how_far_from_target))
        .unwrap();

    return NextBinCollectionDay {
        bin: bin_dates.bin,
        date: closest_day.date,
    };
}

/// Assuming monday is collection day for time being
fn next_collection_date_from(date: NaiveDate) -> NaiveDate {
    let date_weekday = date.weekday();

    let days_from_monday =
        chrono::Duration::days(chrono::Weekday::num_days_from_monday(&date_weekday) as i64);
    let number_of_days_in_week = chrono::Duration::days(7);
    let days_till_monday = number_of_days_in_week - days_from_monday;
    dbg!(days_till_monday.num_days());

    return date + days_till_monday;
}

#[derive(Debug, Clone, Copy)]
pub enum Bin {
    Black,
    Blue,
    Brown,
    Green,
}

impl std::fmt::Display for Bin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_print = match self {
            Bin::Black => "Black",
            Bin::Blue => "Blue",
            Bin::Brown => "Brown",
            Bin::Green => "Green",
        };

        return f.write_str(to_print);
    }
}

#[derive(Debug)]
pub struct BinDates {
    bin: Bin,
    dates: Vec<NaiveDate>,
}

#[derive(Debug)]
struct NextBinCollection {
    bins: Vec<NextBinCollectionDay>,
}

#[derive(Debug, Clone, Copy)]
struct NextBinCollectionDay {
    bin: Bin,
    date: NaiveDate,
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::*;

    #[test]
    fn date_difference_calculation_considers_same_day() {
        let target_date = chrono::Utc::now().date_naive();
        let stuff = calculate_differences_from_date(&[target_date], target_date);

        assert_eq!(stuff.len(), 1);
        assert_eq!(stuff[0].how_far_from_target, chrono::Duration::days(0));
    }

    mod next_collection_date {
        use crate::next_collection_date_from;

        #[test]
        fn it_calculates_next_monday_collection_date() {
            let date = "2023-07-28";
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let next_collection_date = next_collection_date_from(date);

            let expected_collection_date = "2023-07-31";
            let expected_collection_date =
                chrono::NaiveDate::parse_from_str(&expected_collection_date, "%Y-%m-%d").unwrap();

            assert_eq!(next_collection_date, expected_collection_date);
        }

        #[test]
        fn same_day_of_week_calculates_next_week() {
            let date = "2023-07-31";
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let next_collection_date = next_collection_date_from(date);

            let expected_collection_date = "2023-08-07";
            let expected_collection_date =
                chrono::NaiveDate::parse_from_str(&expected_collection_date, "%Y-%m-%d").unwrap();

            assert_eq!(next_collection_date, expected_collection_date);
        }
    }
}
