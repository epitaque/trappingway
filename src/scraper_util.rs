use crate::xiv_util;
use std::str::FromStr;
use std::fs;
use scraper::{Html, Selector};
use simple_error::SimpleError;

pub fn sanitize(input: String) -> String {
    input.replace("\\", "\\\\").replace("#", "#\u{200B}").replace("@", "@\u{200B}").replace("|", "\\|").replace("~", "\\~")
        .replace("[", "\\[").replace("*", "\\*").replace("_", "\\_").replace("`", "\\`").replace(">", "\\>")
}

pub fn get_listings<'a>(html: String) -> Vec<xiv_util::PFListing> {
    let document = Html::parse_document(&html);
    let listing_selector = Selector::parse(".listing").unwrap();

    let elements = document.select(&listing_selector);

    let mut listings = elements.map(|element| {
        let title = element.select(&Selector::parse(".duty").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().next().ok_or(SimpleError::new("parse error"))?.to_owned();
        let author = element.select(&Selector::parse(".creator .text").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().next().ok_or(SimpleError::new("parse error"))?.to_owned();
        let flags = match element.select(&Selector::parse(".description span").unwrap()).next() {
            Some (x) => x.text().next().ok_or(SimpleError::new("parse error"))?.trim_end().to_owned(),
            None => "".to_string()
        };
        // if flags == "" { flags = "[None]".to_string(); }
        let mut description = element.select(&Selector::parse(".description").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.trim_end().to_owned();
        if description == "" { description = "None.".to_string(); }
        let slots = element.select(&Selector::parse(".party .slot").unwrap()).map(|x| {
            let available_jobs = x.value().attr("title").ok_or(SimpleError::new("parse error"))?.split(" ").map(|y| xiv_util::Job::from_str(y)).filter_map(|y| {
                match y {
                    Ok(v) => std::option::Option::Some(v),
                    Err(_) => std::option::Option::None
                }
            }).collect();
            let filled = x.value().attr("class").ok_or(SimpleError::new("parse error"))?.contains("filled");
            Ok(xiv_util::Slot { available_jobs, filled })
        }).filter_map(|w: Result<xiv_util::Slot, SimpleError>| w.ok()).collect::<Vec<_>>();
        let expires_in = element.select(&Selector::parse(".expires .text").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.to_owned();
        let last_updated = element.select(&Selector::parse(".updated .text").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.to_owned();
        let min_ilvl = element.select(&Selector::parse(".middle .stat .value").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.to_owned();
        let data_center = element.value().attr("data-centre").ok_or(SimpleError::new("parse error"))?.to_string();
        let pf_category = element.value().attr("data-pf-category").ok_or(SimpleError::new("parse error"))?.to_string();

        Result::<xiv_util::PFListing, SimpleError>::Ok(xiv_util::PFListing {
            title, 
            author: sanitize(author),
            flags,
            description: sanitize(description),
            slots,
            expires_in,
            last_updated,
            min_ilvl,
            data_center,
            pf_category
        })
    }).filter_map(|w: Result<xiv_util::PFListing, SimpleError>| w.ok()).collect::<Vec<_>>();
    listings.sort_by(|a, b| b.flags.len().partial_cmp(&a.flags.len()).unwrap());

    let mut already_seen_authors: Vec<String> = Vec::new();
    let mut result = Vec::new();
    for listing in listings {
        if already_seen_authors.contains(&listing.author) {
            continue
        }
        already_seen_authors.push(listing.author.to_string());
        result.push(listing)
    }

    result
}

pub async fn get_sample_listings() -> Vec<xiv_util::PFListing> {
    let html = fs::read_to_string("scrape_example.html").expect("Unable to read");
    get_listings(html)
}