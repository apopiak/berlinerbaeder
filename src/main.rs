#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate select;

use reqwest::Client;
use reqwest::Url;

use select::document::Document;
use select::node::{Node};
use select::predicate::*;

error_chain! {
   foreign_links {
       ReqError(reqwest::Error);
       ReqUrlError(reqwest::UrlError);
       IoError(std::io::Error);
   }
}

struct ContainsText<'a>(&'a str);

impl<'a> Predicate for ContainsText<'a> {
    fn matches(&self, node: &Node) -> bool {
        node.text().contains(self.0)
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        println!("usage: baeder <Tag> <Badname>");
    }

    let base_address = Url::parse("https://www.berlinerbaeder.de")?;
    let search_address = base_address.join("/baeder/bad-suche/")?;

    let search_day = args.get(1).expect("expecting at least 2 arguments");
    let empty: String = "".to_owned();
    let filter_term = args.get(2).unwrap_or(&empty);
    let filter = !filter_term.is_empty();

    let client = Client::new();
    let res = client.get(search_address).send()?;
    let baeder_urls: Vec<(String, Url)> = Document::from_read(res)?
        .find(Name("a").and(Attr("href", ())))
        .filter(|n| {
            n.attr("href")
                .expect("we filtered on href already")
                .matches("/baeder/")
                .any(|_| true)
        })
        .filter(|n| {
            if filter {
                n.find(Attr("itemprop", "name").and(ContainsText(filter_term)))
                    .any(|_| true)
            } else {
                n.find(Attr("itemprop", "name")).any(|_| true)
            }
        })
        .filter_map(|n| {
            let link = n.attr("href").expect("we filtered on href already");
            let name = n
                .find(Attr("itemprop", "name"))
                .map(|n| n.text())
                .fold("".to_owned(), |_, t| t);
            base_address.join(link).ok().map(|url| (name, url))
        })
        .collect();

    println!("Offen am {}:", search_day);
    baeder_urls.into_iter().for_each(|(name, url)| {
        client
            .get(url)
            .send()
            .map_err(|e| {
                println!("{:?}", e);
                e
            })
            .map(|res| {
                Document::from_read(res)
                    .map_err(|e| {
                        println!("{:?}", e);
                        e
                    })
                    .map(|d| {
                        println!("{}", name);
                        d.find(Class("tab").and(ContainsText("Badebereich")))
                            .for_each(|swimming| {
                                swimming
                                    .find(Class("day").and(ContainsText(search_day)))
                                    .for_each(|n| {
                                        n.find(Class("opentime")).for_each(|time_node| {
                                            time_node.children().for_each(|time| {
                                                let open_text = time
                                                    .children()
                                                    .filter(|c| c.is(Not(Attr("data-lng", "en"))))
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
                                    })
                            });
                        println!("");
                    })
                    .ok();
            })
            .ok();
    });

    Ok(())
}

quick_main!(run);
