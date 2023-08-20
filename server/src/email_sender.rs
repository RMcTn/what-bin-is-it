use std::error::Error;

use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};

use bin_stuff::{next_collection_date_for_bin, next_collection_date_from, NextBinCollection, User};

pub async fn do_the_stuff(
    users: &[User],
    aws_client: &aws_sdk_sesv2::Client,
    from_email_address: &str,
) -> Result<(), Box<dyn Error>> {
    for person in users {
        println!("Found {:?}", person);
        // TODO: Split out the scraping from the email sending
        // TODO: Keep track who has been sent an email so errors don't cause a retry, meaning
        // multiple emails to the same person or something
        //
        // It's likely if the scraper fails for one person after all attempts, it will fail for all
        // Can pull this out if we want to continue for other people after a scraping error
        // Need to scrape the page from an actual browser. Tried curl/reqwest requests, but submitting
        // the post request would redirect back to the first page. Some sort of request token missing
        // when we do this or something (first step of form submission changes the url).
        let bins = scraper::get_stuff(&person.postcode, &person.address).await?;

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

        let email = build_email_to_send(
            &next_bin_collection,
            &person,
            &aws_client,
            &from_email_address,
        );
        email.send().await?;
        println!("Email sent");
    }
    return Ok(());
}

fn build_email_to_send(
    next_bin_collection: &NextBinCollection,
    person: &User,
    aws_client: &aws_sdk_sesv2::Client,
    from_email_address: &str,
) -> aws_sdk_sesv2::operation::send_email::builders::SendEmailFluentBuilder {
    let mut bin_email_body = String::new();

    let subject = bins_subject(&next_bin_collection);

    for bin_day in &next_bin_collection.bins {
        let s = format!(
            "{} bin is being collected on {}\n",
            bin_day.bin, bin_day.date
        );
        bin_email_body.push_str(&s);
    }

    let destination_email = Destination::builder().to_addresses(&person.email).build();

    let subject_content = Content::builder().data(subject).charset("UTF-8").build();

    let body_content = Content::builder()
        .data(bin_email_body)
        .charset("UTF-8")
        .build();

    let body = Body::builder().text(body_content).build();

    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    dbg!(&msg);

    let email_content = EmailContent::builder().simple(msg).build();
    aws_client
        .send_email()
        .from_email_address(from_email_address)
        .destination(destination_email)
        .content(email_content)
}

fn bins_subject(next_bin_collection: &NextBinCollection) -> String {
    let mut subject = if next_bin_collection.bins.len() == 1 {
        format!("{} bin", next_bin_collection.bins[0].bin)
    } else {
        let mut string = String::new();
        for (i, bin_day) in next_bin_collection.bins.iter().enumerate() {
            if i < next_bin_collection.bins.len() - 1 {
                string.push_str(&format!("{}, ", bin_day.bin));
            } else {
                string.push_str(&format!("{} bins", bin_day.bin));
            }
        }
        string
    };
    subject.push_str(" out tonight");
    return subject;
}

#[cfg(test)]
mod tests {
    use bin_stuff::Bin;
    use bin_stuff::NextBinCollectionDay;
    use std::assert_eq;

    use super::*;

    #[test]
    fn bins_subject_handles_multiple_and_single_bins() {
        let date = "2023-07-31";
        let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let blue_bin = NextBinCollectionDay {
            bin: Bin::Blue,
            date,
        };
        let brown_bin = NextBinCollectionDay {
            bin: Bin::Brown,
            date,
        };

        let mut next_bin_collection = NextBinCollection {
            bins: vec![blue_bin, brown_bin],
        };
        let subject = bins_subject(&next_bin_collection);
        assert_eq!(subject, "Blue, Brown bins out tonight");

        next_bin_collection.bins.pop();

        let subject = bins_subject(&next_bin_collection);
        assert_eq!(subject, "Blue bin out tonight");
    }
}
