use fantoccini::{ClientBuilder, Locator};
use std::error::Error;
use std::time::Duration;
use std::{dbg, format};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // NOTE: Some of the fields get different IDs when submitting each step it seems

    let bins_url = "https://www.northlanarkshire.gov.uk/bin-collection-dates";

    let postcode_input_id = "address-finder-postcode-search-text";
    let postcode = "***REMOVED***";

    let find_address_input_id = "address_finder-postcode-search-button";

    let confirm_button_id = "address_finder_confirm_address_selection";

    let next_button_name = "op";

    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;
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

    let current_url = client.current_url().await?;
    dbg!("Waiting for next button");
    client
        .find(Locator::Css(&format!("input[name={}]", next_button_name)))
        .await?
        .click()
        .await?;
    dbg!("Clicked next button");

    client.wait_for_navigation(Some(current_url)).await?;

    return Ok(());
}
