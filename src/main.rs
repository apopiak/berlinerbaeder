#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate select;

use reqwest::Client;
use reqwest::Url;

use select::document::Document;
use select::node::{Children, Node};
use select::predicate::*;
use select::selection::Selection;

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
    baeder_urls.into_iter().take(1).for_each(|url| {
        client
            .get(url)
            .send()
            .map_err(move |e| {
                println!("{:?}", e);
                e
            })
            .map(move |res| {
                Document::from_read(res)
                    .map_err(move |e| {
                        println!("{:?}", e);
                        e
                    })
                    .map(move |d| {
                        d.find(Class("day").and(|n: &Node| n.text().contains(current_day)))
                            .for_each(|n| {
                                n.find(Class("opentime")).for_each(|time_node| {
                                    time_node.children().for_each(|time| {
                                        let open_text = time
                                            .children()
                                            .filter(|child| {
                                                child.is(|c: &Node| {
                                                    c.attr("data-lng")
                                                        .unwrap_or("")
                                                        .matches("en")
                                                        .count()
                                                        == 0
                                                })
                                            })
                                            .map(|c| c.text().trim().to_owned())
                                            .filter(|c| !c.is_empty())
                                            .collect::<Vec<String>>()
                                            .join(" -> ")
                                            .trim()
                                            .to_owned();
                                        if !open_text.is_empty() {
                                            println!("{}", open_text);
                                        }
                                    })
                                })
                            });
                    })
                    .ok();
            })
            .ok();
    });

    Ok(())
}

quick_main!(run);
