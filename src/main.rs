use chrono::{Datelike, NaiveDate};
use fantoccini::elements::Element;
use fantoccini::{Client, ClientBuilder, Locator};
use std::error::Error;
use std::time::Duration;
use std::{dbg, eprintln, format};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // NOTE: Some of the fields get different IDs when submitting each step it seems

    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;

    fill_out_address_form(&client).await?;

    let black_bins_div = client
        .find(Locator::Css(".waste-type--general-waste"))
        .await?;

    let blue_bins_div = client
        .find(Locator::Css(".waste-type--blue-lidded-recycling-bin"))
        .await?;
    let brown_bins_div = client
        .find(Locator::Css(".waste-type--food-and-garden"))
        .await?;
    let green_bins_div = client
        .find(Locator::Css(
            ".waste-type--glass-metals-plastics-and-cartons",
        ))
        .await?;
    let black_bin_date_elements = black_bins_div.find_all(Locator::Css("p")).await?;
    let blue_bin_date_elements = blue_bins_div.find_all(Locator::Css("p")).await?;
    let brown_bin_date_elements = brown_bins_div.find_all(Locator::Css("p")).await?;
    let green_bin_date_elements = green_bins_div.find_all(Locator::Css("p")).await?;

    // TODO - Clean this up

    let black_bin_dates = get_bin_dates_from_elements(&black_bin_date_elements).await?;
    let blue_bin_dates = get_bin_dates_from_elements(&blue_bin_date_elements).await?;
    let brown_bin_dates = get_bin_dates_from_elements(&brown_bin_date_elements).await?;
    let green_bin_dates = get_bin_dates_from_elements(&green_bin_date_elements).await?;

    let parsed_black_bin_dates = parse_bin_dates(&black_bin_dates);
    let black_bins = BinDates {
        bin: Bin::Black,
        dates: parsed_black_bin_dates,
    };
    let parsed_blue_bin_dates = parse_bin_dates(&blue_bin_dates);
    let blue_bins = BinDates {
        bin: Bin::Blue,
        dates: parsed_blue_bin_dates,
    };
    let parsed_brown_bin_dates = parse_bin_dates(&brown_bin_dates);
    let brown_bins = BinDates {
        bin: Bin::Brown,
        dates: parsed_brown_bin_dates,
    };
    let parsed_green_bin_dates = parse_bin_dates(&green_bin_dates);
    let green_bins = BinDates {
        bin: Bin::Green,
        dates: parsed_green_bin_dates,
    };

    let bins = [black_bins, blue_bins, brown_bins, green_bins];

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

    dbg!(next_bin_collection);

    return Ok(());
}

async fn fill_out_address_form(client: &Client) -> Result<(), Box<dyn Error>> {
    let bins_url = "https://www.northlanarkshire.gov.uk/bin-collection-dates";

    let postcode_input_id = "address-finder-postcode-search-text";
    let postcode = "***REMOVED***";

    let find_address_input_id = "address_finder-postcode-search-button";

    let confirm_button_id = "address_finder_confirm_address_selection";

    let next_button_name = "op";

    client.goto(bins_url).await?;

    dbg!("Waiting for cookie confirmation");
    client
        .find(Locator::Css(".cb-enable"))
        .await?
        .click()
        .await?;
    dbg!("Clicked cookie");

    dbg!("Little sleep for page load");
    tokio::time::sleep(Duration::from_secs(1)).await;
    dbg!("Waiting for postcode input box");
    let postcode_input = client.find(Locator::Id(postcode_input_id)).await?;

    postcode_input.click().await?;
    dbg!("Clicked postcode input box");
    // Enter key doesn't submit this form
    postcode_input.send_keys(postcode).await?;
    // TODO - Check the input box to make sure a value is selected (or rely on the confirm part
    // after?)

    dbg!("Waiting for find address button");
    client
        .find(Locator::Id(find_address_input_id))
        .await?
        .click()
        .await?;
    dbg!("Submitted find address button");
    dbg!("Little sleep for page load");
    tokio::time::sleep(Duration::from_secs(1)).await;

    dbg!("Waiting for address drop down");
    let address_drop_down = client.find(Locator::Css("select.form-select")).await?;
    address_drop_down.click().await?;
    dbg!("Clicked address drop down");

    address_drop_down.send_keys("***REMOVED***").await?;
    // TODO: Is Enter needed here?
    dbg!("Waiting for confirm address button");
    client
        .find(Locator::Id(confirm_button_id))
        .await?
        .click()
        .await?;
    dbg!("Clicked confirm address button");
    // Ignoring successful address lookup check

    dbg!("Waiting for next button");
    client
        .find(Locator::Css(&format!("input[name={}]", next_button_name)))
        .await?
        .click()
        .await?;
    dbg!("Clicked next button");

    dbg!("Waiting for next page");
    client
        .wait()
        .for_element(Locator::Css(".bin-collection-dates-container"))
        .await?;

    dbg!("On next page");
    return Ok(());
}

async fn get_bin_dates_from_elements(
    elements: &Vec<Element>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut bin_dates = Vec::new();
    for element in elements {
        bin_dates.push(element.text().await?);
    }

    return Ok(bin_dates);
}

fn parse_bin_dates(bin_date_strings: &[String]) -> Vec<NaiveDate> {
    let mut parsed_dates = Vec::new();
    for date in bin_date_strings {
        match chrono::NaiveDate::parse_from_str(&date, "%d %B %Y") {
            Ok(parsed_date) => parsed_dates.push(parsed_date),
            Err(e) => eprintln!("Error parsing {}: {}", date, e),
        }
    }
    return parsed_dates;
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
enum Bin {
    Black,
    Blue,
    Brown,
    Green,
}

#[derive(Debug)]
struct BinDates {
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
