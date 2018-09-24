#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate select;

use reqwest::Client;
use reqwest::Url;

use select::document::Document;
use select::predicate::*;

error_chain! {
   foreign_links {
       ReqError(reqwest::Error);
       ReqUrlError(reqwest::UrlError);
       IoError(std::io::Error);
   }
}

fn run() -> Result<()> {
    let base_address = Url::parse("https://www.berlinerbaeder.de")?;
    let search_address = base_address.join("/baeder/bad-suche/")?;
    let client = Client::new();
    let res = client.get(search_address).send()?;

    let baeder_urls: Vec<Url> = Document::from_read(res)?
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .filter(|l| l.matches("/baeder/").count() > 0)
        .skip(1)
        .filter_map(|l| base_address.join(l).ok())
        // .map(|x| { println!("{:?}", x); x })
        .collect();

    let current_day = "Sonntag";
    baeder_urls.into_iter().for_each(|url| {
        client.get(url).send().map_err(|e| {println!("{:?}", e); e}).map(|res| {
            Document::from_read(res).map_err(|e| {println!("{:?}", e); e}).map(|d| {
                d.find(Class("day")).filter(|n| n.into_selection()).for_each(|x| println!("{:?}", x))
            }).ok();
        }).ok();

    });

    Ok(())
}

quick_main!(run);
