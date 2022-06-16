use crate::xiv_util;
use std::str::FromStr;
use std::fs;
use scraper::{Html, Selector};
use simple_error::SimpleError;

// pub fn get_listings_aux(html: String) -> Result<Vec<xiv_util::PFListing>, SimpleError> {
//     match get_listings(html) {
//         Ok(r) => {
//             Ok(r)
//         } Err(_) => {
//             Err(SimpleError::new("Error parsing html"))
//         }
//     }
// }

pub fn get_listings<'a>(html: String) -> Vec<xiv_util::PFListing> {
    let document = Html::parse_document(&html);

    // let listing_selector = match Selector::parse(".listing") {
    //     Ok(v) => v,
    //     Err(_) => { return std::result::Result::Err(SimpleError::new("cannot parse .listing")) }
    // };

    let listing_selector = Selector::parse(".listing").unwrap();

    let elements = document.select(&listing_selector);

    elements.map(|element| {
        let title = element.select(&Selector::parse(".duty").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().next().ok_or(SimpleError::new("parse error"))?.to_owned();
        let author = element.select(&Selector::parse(".creator .text").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().next().ok_or(SimpleError::new("parse error"))?.to_owned();
        let flags = match element.select(&Selector::parse(".description span").unwrap()).next() {
            Some (x) => x.text().next().ok_or(SimpleError::new("parse error"))?.trim_end().to_owned(),
            None => "".to_string()
        };
        
        let description = element.select(&Selector::parse(".description").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.trim_end().to_owned();
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
        let time_remaining = element.select(&Selector::parse(".expires .text").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.to_owned();
        let min_ilvl = element.select(&Selector::parse(".middle .stat .value").unwrap()).next().ok_or(SimpleError::new("parse error"))?.text().last().ok_or(SimpleError::new("parse error"))?.to_owned();
        let data_center = element.value().attr("data-centre").ok_or(SimpleError::new("parse error"))?.to_string();
        let pf_category = element.value().attr("data-pf-category").ok_or(SimpleError::new("parse error"))?.to_string();

        Result::<xiv_util::PFListing, SimpleError>::Ok(xiv_util::PFListing {
            title, 
            author,
            flags,
            description,
            slots,
            time_remaining,
            min_ilvl,
            data_center,
            pf_category
        })
    }).filter_map(|w: Result<xiv_util::PFListing, SimpleError>| w.ok()).collect::<Vec<_>>()
}

pub async fn get_sample_listings() -> Vec<xiv_util::PFListing> {
    let html = fs::read_to_string("scrape_example.html").expect("Unable to read");
    get_listings(html)
}

#[allow(dead_code)]
async fn test() {
    // let html = reqwest::get("https://bloomberg.com/")
    //     .await?
    //     .text()
    //     .await?;
    let listings = get_sample_listings();
    println!("A listing: {}", listings.await[0].to_string());



    // let forever = task::spawn(async {
    //     let mut interval = time::interval(Duration::from_millis(1000));

    //     loop {
    //         interval.tick().await;
    //         do_something().await;
    //     }
    // });

    // forever.await;
}
