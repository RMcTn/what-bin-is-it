use chrono::{Datelike, NaiveDate};

/// Date returned will be 1 week from target_date if collection_day is the same day as target_date
/// Assumption is that we don't request the collection date on the same day
pub fn next_collection_date_from(target_date: NaiveDate, collection_day: chrono::Weekday) -> NaiveDate {
    let mut days_until_collection = 1; //
    let mut target_day = target_date.weekday();
    while target_day.succ() != collection_day {
        target_day = target_day.succ();
        days_until_collection += 1;
    }
    return target_date + chrono::Duration::days(days_until_collection);
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
        use chrono::Weekday;

        use crate::next_collection_date_from;

        #[test]
        fn it_calculates_next_collection_date_for_given_weekday() {
            let date = "2023-07-28";
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let next_collection_date = next_collection_date_from(date, Weekday::Mon);

            let expected_collection_date = "2023-07-31";
            let expected_collection_date =
                chrono::NaiveDate::parse_from_str(&expected_collection_date, "%Y-%m-%d").unwrap();

            assert_eq!(next_collection_date, expected_collection_date);

            let next_collection_date = next_collection_date_from(date, Weekday::Wed);

            let expected_collection_date = "2023-08-02";
            let expected_collection_date =
                chrono::NaiveDate::parse_from_str(&expected_collection_date, "%Y-%m-%d").unwrap();

            assert_eq!(next_collection_date, expected_collection_date);
        }

        #[test]
        fn same_day_of_week_calculates_next_week() {
            let date = "2023-07-31";
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let next_collection_date = next_collection_date_from(date, Weekday::Mon);

            let expected_collection_date = "2023-08-07";
            let expected_collection_date =
                chrono::NaiveDate::parse_from_str(&expected_collection_date, "%Y-%m-%d").unwrap();

            assert_eq!(next_collection_date, expected_collection_date);
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
