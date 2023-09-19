use chrono::{Datelike, NaiveDate};

/// Assuming monday is collection day for time being
pub fn next_collection_date_from(date: NaiveDate) -> NaiveDate {
    let date_weekday = date.weekday();
    // TODO: Remove monday collection day assumption

    let days_from_monday =
        chrono::Duration::days(chrono::Weekday::num_days_from_monday(&date_weekday) as i64);
    let number_of_days_in_week = chrono::Duration::days(7);
    let days_till_monday = number_of_days_in_week - days_from_monday;
    dbg!(days_till_monday.num_days());

    return date + days_till_monday;
}

pub fn next_collection_date_for_bin(
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

pub fn next_bin_collection_date(bins: &[BinDates], target_date: NaiveDate) -> NextBinCollection {
    let next_collection_date = next_collection_date_from(target_date);

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

    return next_bin_collection;
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub bin: Bin,
    pub dates: Vec<NaiveDate>,
}

#[derive(Debug)]
pub struct NextBinCollection {
    pub bins: Vec<NextBinCollectionDay>,
}

#[derive(Debug, Clone, Copy)]
pub struct NextBinCollectionDay {
    pub bin: Bin,
    pub date: NaiveDate,
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
    dbg!(target_date);
    dbg!(&dates);
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

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn date_difference_calculation_considers_same_day() {
        let target_date = chrono::Utc::now().date_naive();
        let stuff = calculate_differences_from_date(&[target_date], target_date);

        assert_eq!(stuff.len(), 1);
        assert_eq!(stuff[0].how_far_from_target, chrono::Duration::days(0));
    }

    mod next_collection_date {
        use crate::{Bin, BinDates, next_bin_collection_date, next_collection_date_from};

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

        #[test]
        fn it_returns_bin_with_nearest_collection_date_from_given_date() {
            let date = "2023-07-31";
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let yesterday = date - chrono::Duration::days(1);
            // TODO: Not happy with the dates in this test since our collection date is always
            //  assumed to be monday
            let six_days_from_now = date + chrono::Duration::days(6);
            let one_week_from_now = date + chrono::Duration::days(7);

            let green_date = vec![one_week_from_now];
            let green_bin_date = BinDates {
                bin: Bin::Green,
                dates: green_date,
            };
            let black_date = vec![one_week_from_now];
            let black_bin_date = BinDates {
                bin: Bin::Black,
                dates: black_date,
            };
            let blue_date = vec![one_week_from_now];
            let blue_bin_date = BinDates {
                bin: Bin::Blue,
                dates: blue_date,
            };
            let bin_dates = [black_bin_date, blue_bin_date, green_bin_date];
            let next_bins_date = next_bin_collection_date(&bin_dates, date);
            let bins_to_be_collected: Vec<Bin> = next_bins_date.bins.iter().map(|bins| bins.bin).collect();
            let expected_bins_to_be_collected = vec![Bin::Black, Bin::Blue];
            assert!(bins_to_be_collected.iter().eq(expected_bins_to_be_collected.iter()));
        }
    }
}

#[derive(Debug)]
// TODO: Not where I want to put this, but it's convenient for now.
// Can't import this struct from the server binary crate into the email_sender crate so this is what works right now
pub struct User {
    pub _id: i64,
    // TODO: Better types for these with some validation?
    pub email: String,
    pub postcode: String,
    pub address: String,
}
