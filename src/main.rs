use std::error::Error;
use structopt::StructOpt;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User {
    name: String,
    handle: String,
    email: Option<String>,
    phonenumber: Option<String>,
    picture_url: Option<String>,
    status: SlackStatus
}

#[derive(Serialize,Debug)]
enum SlackStatus {
    Active,
    Away,
    DnD,
    Offline
}

#[derive(Debug)]
enum OutputTypes {
    JSON,
    HTML,
    Csv,
    Markdown,
    Table
}

fn parse_output(output: &str) -> Result<OutputTypes, String> {
    match output {
            "table" => Ok(OutputTypes::Table),
            "json" => Ok(OutputTypes::JSON),
            "html" => Ok(OutputTypes::HTML),
            "csv" => Ok(OutputTypes::Csv),
            "markdown" => Ok(OutputTypes::Markdown),
            "md" => Ok(OutputTypes::Markdown),
            _ => Err(format!("Invalid Output Type: {}", output))
        }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "slackline")]
struct CLI {
    /// Slack API Key 
    #[structopt(short = "k", long = "key")]
    api_key: Option<String>,

    /// Limit Search to a single Slack channel
    #[structopt(short = "c", long = "channel")]
    channel: Option<String>,

    #[structopt(short = "o", long = "output", parse(try_from_str = "parse_output"), default_value = "table")]
    /// Output format
    output: OutputTypes
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut cli_opts = CLI::from_args();

    cli_opts.api_key = Some(match cli_opts.api_key {
        Some(api_key) => api_key,
        None => {
            std::env::var("SLACK_API_KEY")?
        }
    });
    
    
    let client = slack_api::default_client()?;
    let token = cli_opts.api_key.unwrap();

    if let Some(channel) = cli_opts.channel {
        let chan_req = slack_api::channels::ListRequest {
            exclude_archived: Some(true),
            exclude_members: Some(false)

        };
        let channels = slack_api::channels::list(&client, &token, &chan_req)?;

    }

    let users_req = slack_api::users::ListRequest {
        presence: None
    };

    let users = slack_api::users::list(&client, &token, &users_req)?;

    if let Some(users) = users.members {
        let parsed_users = users.par_iter().map(|mut user| {
            let user = user.to_owned();
            let profile = user.profile.unwrap();
            User {
                name: user.real_name.unwrap(),
                handle: user.name.unwrap(),
                email: profile.email,
                phonenumber: profile.phone,
                picture_url: profile.image_512,
                status: SlackStatus::Offline
            }
        }).collect::<Vec<User>>();

        match cli_opts.output {
            OutputTypes::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for user in parsed_users {
                    wtr.serialize(user)?;
                }
                let data = String::from_utf8(wtr.into_inner()?)?;
                println!("{}", data);
            },
            OutputTypes::HTML => {
                println!("<html><body>");
                println!("<h1>Slack Team Directory</h1>");
                for user in parsed_users {
                    let phone = match user.phonenumber {
                        Some(phone) => phone,
                        None => String::from("N/A")
                    };
                    let email = match user.email {
                        Some(phone) => phone,
                        None => String::from("N/A")
                    };
                    let picture = match user.picture_url {
                        Some(phone) => phone,
                        None => String::from("N/A")
                    };

                    println!("  <div>");
                    println!("    <h2>{}</h2>", user.name);
                    println!("    <img src=\"{}\" style='height: 48px; width: auto;'></img>", picture);
                    println!("    <h3>Slack: {}</h3>", user.handle);
                    println!("    <h3>Phone: {}</h3>", phone);
                    println!("    <h3>Email: <a src=\"mailto:{}\">{}</a></h3>", email, email);
                    println!("  </div>");
                }
                println!("</html></body>");
            },
            OutputTypes::JSON => {
                println!("{}", serde_json::to_string_pretty(&parsed_users)?);
            },
            OutputTypes::Markdown => {},
        OutputTypes::Table =>  {
        println!( "{0: <10} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10}", "Name", "handle", "phone", "email", "status", "picture url");

            for user in parsed_users {

                let phone = match user.phonenumber {
                    Some(phone) => phone,
                    None => String::from("N/A")
                };
                let email = match user.email {
                    Some(phone) => phone,
                    None => String::from("N/A")
                };
                let picture = match user.picture_url {
                    Some(phone) => phone,
                    None => String::from("N/A")
                };

                println!("{0: <18} | {1: <10} | {2: <15} | {3: <3} | {4: <10} | {5: <10}", user.name, user.handle, phone, email, "N/A", picture);

            }

            }

        }
    }


    Ok(())
}
