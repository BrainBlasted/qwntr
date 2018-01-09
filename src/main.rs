extern crate termion;
extern crate liner;
extern crate qwant_api;
extern crate locate_locale;
extern crate regex;
extern crate webbrowser;

use termion::{style, color};
use liner::Context;
use qwant_api::{APIResponse,SearchType,Item};
#[allow(unused_imports)]
use regex::Regex;

use std::io::ErrorKind;

// ID to pass to Qwant API as "&t=APP_ID"
const APP_ID: &str = "qwntr";

fn main() {
    // Establish locale of user
    let locale = locate_locale::user();

    let mut prompt = Context::new();

    let mut stored_items: Vec<Item> = vec!();

    let mut stored_response: Option<APIResponse> = None;

    loop {
        let prompt_str = format!("{invert}{bold} qwntr (/? for help, /q to quit) {clear}: ",
            invert = style::Invert,
            bold = style::Bold,
            clear = style::Reset,
            );

        // Convert prompt_str to &str. This is required because read_line
        // does not take std::string::String, but &str.
        // Doing format!(...).as_str() is not possible because the formatted
        // value cannot live without a let binding.
        let prompt_str = prompt_str.as_str();

        let search_str: Option<String> = match prompt.read_line(prompt_str, &mut |_| {}) {
            Ok(s) => Some(s),
            Err(e) => {
                match e.kind() {
                    // Break on Ctrl+C or Ctrl+D,
                    // else continue the loop.
                    ErrorKind::Interrupted => { exit(); }
                    ErrorKind::UnexpectedEof => { exit(); }
                    _ => continue,
                }
                None // For the compiler
            }
        };

        // Since the loop continues if search returns None(it never actually reaches None),
        // unwrap search.
        let mut search_str: String = search_str.unwrap();
        prompt.history.push(liner::Buffer::from(search_str.clone())).unwrap();
        search_str = search_str.trim().to_string();
        if search_str.is_empty() {continue}
        if search_str == "/q" { exit() }
        if search_str.starts_with("/o") {
            open_result(&search_str, &stored_items);
            continue;
        }
        if stored_response.is_some() && search_str == "/n" {
            // Will this be resource intensive to keep cloning like
            // this?
            let mut new_resp = stored_response.clone().unwrap();
            new_resp.next_page();
            println!("offset is {}", new_resp.clone().data.unwrap().query.unwrap().offset);
            new_resp = new_resp.clone();

            let mut new_items = new_resp.clone().data.unwrap().result.items;
            stored_items.clone_from(&mut new_items);
            display_response(&mut new_resp);
            continue;
        } else if search_str == "/n" {
            println!("No previous search");
        }

        // Send query to API
        let safe_search = false;
        let mut response = match APIResponse::new(&search_str,
                                                  &SearchType::Web,
                                                  safe_search,
                                                  &locale,
                                                  &APP_ID.to_string())
        {
            Some(resp) => resp,
            None => {continue;},
        };

        stored_response = Some(response.clone());

        let items = match response.clone().data {
            Some(dat) => dat.result.items,
            None => {
                println!("No data received");
                continue;
            },
        };

        stored_items.clone_from(&items);

        display_response(&mut response);
    }
}

fn display_response(resp: &mut APIResponse) {
    // let title_regex =
    let items: Vec<Item> = resp.clone().data.unwrap().result.items;
    for (i, mut item) in items.into_iter().enumerate() {
        item.strip_html();
        let domain = item.url;
        println!("{blue}({}) {bold}{green}{}{style_reset} {orange}[{}]{reset}",
                 i+1,
                 item.title,
                 domain,
                 blue = color::Fg(color::Blue),
                 green = color::Fg(color::Green),
                 bold = style::Bold,
                 style_reset = style::Reset,
                 orange = color::Fg(color::Rgb(255, 165, 0)),
                 reset = color::Fg(color::Reset));
        println!("{}\n", item.desc);
    }
}

// Method that handles opening results passed
// with /o.
fn open_result(res: &str, items: &Vec<Item>) {
    if items.is_empty() {
        println!("No results stored");
        return;
    }

    let split = res.split_whitespace();
    // Parses each index in the split and
    // opens it in the web browser.
    // This uses the same browser-opening
    // choices as webbrowser-rs at
    // https://github.com/amodm/webbrowser-rs
    for (i, s) in split.enumerate() {
        if i != 0 {
            let index: usize = match s.parse() {
                Ok(ind) => ind,
                Err(_) => {
                    println!("{red}Could not parse inputs{reset}",
                             red = color::Fg(color::Red),
                             reset = color::Fg(color::Reset));
                    continue;
                }
            };

            // Skip over out-of-bounds and overflow-causing
            // inputs
            if index > items.len() || index == 0 {continue;}
            match webbrowser::open(&items[index - 1].url) {
                Ok(_) => (),
                Err(e) => {
                    println!("{red}Could not open browser.",
                             red = color::Fg(color::Red));
                    println!("{:?}{reset}",
                             e,
                             reset = color::Fg(color::Reset));
                }
            }
        }
    }
}

// A nice method to handle gracefull exits of the program.
fn exit() {
    println!("{green}Goodbye!{reset}",
             green = color::Fg(color::Green),
             reset = color::Fg(color::Reset));
    std::process::exit(0);
}
