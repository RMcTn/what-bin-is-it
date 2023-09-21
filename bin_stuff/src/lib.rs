use chrono::{Datelike, NaiveDate};

/// Date returned will be 1 week from target_date if collection_day is the same day as target_date
/// Assumption is that we don't request the collection date on the same day
pub fn next_collection_date_from(
    target_date: NaiveDate,
    collection_day: chrono::Weekday,
) -> NaiveDate {
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
) -> Option<NextBinCollectionDay> {
    let diffs = calculate_differences_from_date(&bin_dates.dates, target_date)?;

    let closest_day = diffs
        .iter()
        .min_by(|x, y| x.how_far_from_target.cmp(&y.how_far_from_target))
        .unwrap();

    return Some(NextBinCollectionDay {
        bin: bin_dates.bin,
        date: closest_day.date,
    });
}

pub fn next_bin_collection_date(
    bins: &[BinDates],
    target_date: NaiveDate,
    target_weekday: chrono::Weekday,
) -> NextBinCollection {
    let next_collection_date = next_collection_date_from(target_date, target_weekday);

    dbg!(next_collection_date);
    let mut next_collection_day_for_bins = Vec::new();
    for bin in bins {
        let next_day = next_collection_date_for_bin(&bin, next_collection_date);
        if next_day.is_none() {
            continue;
        }
        match next_day {
            Some(day) => next_collection_day_for_bins.push(day),
            None => continue,
        }
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

    return NextBinCollection {
        bins: closest_bin_days,
    };
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
/// None if the all dates are from before the target_date
fn calculate_differences_from_date(
    dates: &[NaiveDate],
    target_date: NaiveDate,
) -> Option<Vec<TimeFromTarget>> {
    let differences: Vec<TimeFromTarget> = dates
        .iter()
        .map(|date| TimeFromTarget {
            _target_date: target_date,
            date: *date,
            how_far_from_target: *date - target_date,
        })
        .filter(|time_from_target| time_from_target.how_far_from_target.num_seconds() >= 0)
        .collect();

    // TODO: Add test for this case?
    if differences.is_empty() {
        return None;
    }
    return Some(differences);
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn date_difference_calculation_considers_same_day() {
        let target_date = chrono::Utc::now().date_naive();
        let stuff = calculate_differences_from_date(&[target_date], target_date).unwrap();

        assert_eq!(stuff.len(), 1);
        assert_eq!(stuff[0].how_far_from_target, chrono::Duration::days(0));
    }

    mod next_collection_date {
        use chrono::{Datelike, Weekday};

        use crate::{next_bin_collection_date, next_collection_date_from, Bin, BinDates};

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

        #[test]
        fn next_bin_collection_date_test() {
            let today = chrono::Utc::now().date_naive();
            let one_week_from_today = today + chrono::Duration::days(7);

            let green_bin_date = BinDates {
                bin: Bin::Green,
                dates: vec![today],
            };
            let blue_bin_date = BinDates {
                bin: Bin::Blue,
                dates: vec![one_week_from_today],
            };
            let black_bin_date = BinDates {
                bin: Bin::Black,
                dates: vec![one_week_from_today],
            };

            let expected_bins = [blue_bin_date.bin, black_bin_date.bin];
            let expected_bin_dates = [blue_bin_date.dates[0], black_bin_date.dates[0]];
            let bins = [green_bin_date, blue_bin_date, black_bin_date];

            let next_bin_collection = next_bin_collection_date(&bins, today, today.weekday());
            let bins_to_be_collected: Vec<_> =
                next_bin_collection.bins.iter().map(|bin| bin.bin).collect();
            assert!(bins_to_be_collected.iter().eq(expected_bins.iter()));
            let bins_collected_on: Vec<_> = next_bin_collection
                .bins
                .iter()
                .map(|bin| bin.date)
                .collect();
            assert!(bins_collected_on.iter().eq(expected_bin_dates.iter()));
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
