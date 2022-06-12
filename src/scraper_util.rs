use crate::xiv_util;
use std::str::FromStr;
use std::fs;
use scraper::{Html, Selector};

pub fn get_listings(html: String) -> Vec<xiv_util::PFListing> {
    let document = Html::parse_document(&html);

    let listing_selector = Selector::parse(".listing").unwrap();

    let elements = document.select(&listing_selector);

    elements.map(|element| {
        let title = element.select(&Selector::parse(".duty").unwrap()).next().unwrap().text().collect::<Vec<_>>()[0].to_owned();
        let author = element.select(&Selector::parse(".creator .text").unwrap()).next().unwrap().text().collect::<Vec<_>>()[0].to_owned();
        let description = element.select(&Selector::parse(".description").unwrap()).next().unwrap().text().collect::<Vec<_>>()[0].to_owned();
        let slots = element.select(&Selector::parse(".party .slot").unwrap()).map(|x| {
            let available_roles = x.value().attr("title").unwrap().split(" ").map(|y| xiv_util::Job::from_str(y)).filter_map(|y| {
                match y {
                    Ok(v) => std::option::Option::Some(v),
                    Err(_) => std::option::Option::None
                }
            }).collect();
            let filled = x.value().attr("class").unwrap().contains("filled");
            xiv_util::Slot { available_roles, filled }
        }).collect::<Vec<_>>();
        let time_remaining = 5u32;
        let min_ilvl = 5u32;
        let data_center = element.value().attr("data-centre").unwrap().to_string();
        let pf_category = element.value().attr("data-pf-category").unwrap().to_string();

        let listing = xiv_util::PFListing {
            title, 
            author,
            description, 
            slots,
            time_remaining,
            min_ilvl,
            data_center,
            pf_category
        };

        // println!("PF title: {}", listing.to_string());
        listing
    }).collect()
}

pub fn get_sample_listings() -> Vec<xiv_util::PFListing> {
    let html = fs::read_to_string("scrape_example.html").expect("Unable to read");
    get_listings(html)
}

async fn test() {
    // let html = reqwest::get("https://bloomberg.com/")
    //     .await?
    //     .text()
    //     .await?;
    let listings = get_sample_listings();
    println!("A listing: {}", listings[0].to_string());



    // let forever = task::spawn(async {
    //     let mut interval = time::interval(Duration::from_millis(1000));

    //     loop {
    //         interval.tick().await;
    //         do_something().await;
    //     }
    // });

    // forever.await;
}
