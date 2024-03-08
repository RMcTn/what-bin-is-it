use anyhow::Error;

use chrono::NaiveDate;
use fantoccini::elements::Element;
use fantoccini::wd::Capabilities;
use fantoccini::{Client, ClientBuilder, Locator};

use bin_stuff::{Bin, BinDates};
use log::{error, info};

pub async fn get_stuff(
    postcode: &str,
    address: &str,
    driver_url: Option<String>,
) -> Result<Vec<BinDates>, Error> {
    let mut capabilities = Capabilities::new();
    let options = serde_json::json!({ "args": ["--headless"] });
    capabilities.insert("moz:firefoxOptions".to_string(), options);

    info!("Attempting to connect to webdriver client");
    let default_driver_url = "http://127.0.0.1:4444".to_string();
    if driver_url.is_none() {
        info!(
            "No driver_url provided. Defaulting to {}",
            default_driver_url
        );
    }
    let client = ClientBuilder::native()
        .capabilities(capabilities)
        .connect(&driver_url.unwrap_or("http://127.0.0.1:4444".to_string()))
        .await?;
    info!("Got webdriver client");

    // NOTE: Some of the fields get different IDs when submitting each step it seems
    let max_attempts = 3;
    let mut attempts = 0;
    while attempts < max_attempts {
        info!("Visiting bin page");
        info!("Attempt {}/{}", attempts, max_attempts);
        match fill_out_address_form(&client, postcode, address).await {
            Ok(_) => break,
            Err(e) => {
                attempts += 1;
                error!("{}", e);

                if attempts == max_attempts {
                    error!("Reached max attempt limit");
                    return Err(e);
                }
            }
        }
    }

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

    let bins = vec![black_bins, blue_bins, brown_bins, green_bins];

    return Ok(bins);
}

async fn fill_out_address_form(
    client: &Client,
    postcode: &str,
    address: &str,
) -> Result<(), Error> {
    let bins_url = "https://www.northlanarkshire.gov.uk/bin-collection-dates";

    let postcode_input_id = "address-finder-postcode-search-text";

    let find_address_input_id = "address_finder-postcode-search-button";

    let confirm_button_id = "address_finder_confirm_address_selection";

    let next_button_name = "op";

    info!("Going to site");
    client.goto(bins_url).await?;

    info!("Waiting for cookie confirmation");
    client
        .find(Locator::Css(".cb-enable"))
        .await?
        .click()
        .await?;
    info!("Clicked cookie");

    info!("Little sleep for page load");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    info!("Waiting for postcode input box");
    let postcode_input = client.find(Locator::Id(postcode_input_id)).await?;

    postcode_input.click().await?;
    info!("Clicked postcode input box");
    // Enter key doesn't submit this form
    postcode_input.send_keys(postcode).await?;
    // TODO - Check the input box to make sure a value is selected (or rely on the confirm part
    // after?)

    info!("Waiting for find address button");
    client
        .find(Locator::Id(find_address_input_id))
        .await?
        .click()
        .await?;
    info!("Submitted find address button");
    info!("Little sleep for page load");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    info!("Waiting for address drop down");
    let address_drop_down = client.find(Locator::Css("select.form-select")).await?;
    address_drop_down.click().await?;
    info!("Clicked address drop down");

    address_drop_down.send_keys(address).await?;
    // TODO: Is Enter needed here?
    info!("Waiting for confirm address button");
    client
        .find(Locator::Id(confirm_button_id))
        .await?
        .click()
        .await?;
    info!("Clicked confirm address button");
    // Ignoring successful address lookup check

    info!("Waiting for next button");
    client
        .find(Locator::Css(&format!("input[name={}]", next_button_name)))
        .await?
        .click()
        .await?;
    info!("Clicked next button");

    info!("Waiting for next page");
    client
        .wait()
        .for_element(Locator::Css(".bin-collection-dates-container"))
        .await?;

    info!("On next page");
    return Ok(());
}

async fn get_bin_dates_from_elements(elements: &Vec<Element>) -> Result<Vec<String>, Error> {
    let mut bin_dates = Vec::new();
    for element in elements {
        bin_dates.push(element.text().await?);
    }

    return Ok(bin_dates);
}

fn parse_bin_dates(bin_date_strings: &[String]) -> Vec<NaiveDate> {
    let mut parsed_dates = Vec::new();
    for date in bin_date_strings {
        match chrono::NaiveDate::parse_from_str(date, "%d %B %Y") {
            Ok(parsed_date) => parsed_dates.push(parsed_date),
            Err(e) => eprintln!("Error parsing {}: {}", date, e),
        }
    }
    return parsed_dates;
}
