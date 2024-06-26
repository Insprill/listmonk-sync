use anyhow::Context;
use itertools::Itertools;
use log::{error, info, LevelFilter};
use reqwest::{
    multipart::{Form, Part},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use std::{env, error::Error, fs::File, io::BufWriter, time::Duration};
use tokio::{task, time};

const SQUARE_VERSION: &str = "2023-01-19";

#[derive(Deserialize, Debug)]
struct Config {
    /// How often the program should run, in seconds.
    run_every: u64,

    /// The domain of your listmonk instance. This should not include 'https://' or a path.
    listmonk_domain: String,

    /// The ID of the listmonk list to add the subscribers to. Can be found at the top of the edit modal.
    listmonk_list_ids: Vec<u64>,

    /// Whether the imported customers should be marked as confirmed.
    listmonk_confirmation: bool,

    /// Whether subscribers should be overwritten.
    listmonk_overwrite: bool,
}

#[derive(Debug)]
struct Credentials {
    square_api_token: String,
    listmonk_username: String,
    listmonk_password: String,
}

#[derive(Deserialize, Debug)]
struct SquareCustomer {
    email_address: Option<String>,
    preferences: SquareCustomerPrefrences,
    given_name: Option<String>,
    family_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SquareCustomerPrefrences {
    email_unsubscribed: bool,
}

#[derive(Deserialize, Debug)]
struct SquareCustomersResponse {
    customers: Vec<SquareCustomer>,
    cursor: Option<String>,
}

#[derive(Serialize, Debug)]
struct ListmonkSubscriber {
    email: String,
    name: String,
    attributes: String,
    #[serde(skip_serializing)]
    subscribed: bool,
}

#[derive(Serialize, Debug)]
struct ListmonkImport {
    mode: &'static str,
    delim: &'static str,
    subscription_status: &'static str,
    lists: Vec<u64>,
    overwrite: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let config_file =
        File::open("config.json").with_context(|| "Failed to open config.json! Does it exist?")?;
    let config: Config = serde_json::from_reader(config_file).with_context(|| {
        "Failed to deserialize config. Does it have syntax errors or missing fields?"
    })?;

    let creds = Credentials {
        square_api_token: env::var("SQUARE_API_TOKEN")
            .with_context(|| "Missing the SQUARE_API_TOKEN environment variable!")?,
        listmonk_username: env::var("LISTMONK_USER")
            .with_context(|| "Missing the LISTMONK_USER environment variable!")?,
        listmonk_password: env::var("LISTMONK_PASSWORD")
            .with_context(|| "Missing the LISTMONK_PASSWORD environment variable!")?,
    };

    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(config.run_every));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            info!("Syncing...");
            if let Err(err) = run(&config, &creds).await {
                error!("Failed to sync! {:?}", err);
            } else {
                info!("Sync complete.")
            }
        }
    });

    Ok(forever.await?)
}

async fn run(config: &Config, creds: &Credentials) -> Result<(), Box<dyn Error>> {
    let subscribers: Vec<ListmonkSubscriber> = get_square_customers(creds)
        .await?
        .into_iter()
        .filter(|c| c.email_address.is_some())
        .sorted_by_key(|c| {
            (
                c.given_name.is_some() || c.family_name.is_some(),
                !c.preferences.email_unsubscribed,
            )
        })
        .unique_by(|c| c.email_address.as_ref().unwrap().to_lowercase()) // Unwrap validated in filter
        .map(|customer| ListmonkSubscriber {
            email: customer.email_address.unwrap(),
            name: format!(
                "{} {}",
                customer.given_name.unwrap_or("Customer".to_string()),
                customer.family_name.unwrap_or_default()
            )
            .trim()
            .to_string(),
            attributes: String::new(),
            subscribed: !customer.preferences.email_unsubscribed,
        })
        .collect();

    info!("Found {} unique Square customers", subscribers.len());

    for mode in [("subscribe", true), ("blocklist", false)] {
        let import = ListmonkImport {
            mode: mode.0,
            delim: ",",
            subscription_status: if config.listmonk_confirmation {
                "confirmed"
            } else {
                "unconfirmed"
            },
            lists: config.listmonk_list_ids.clone(),
            overwrite: config.listmonk_overwrite,
        };

        let subs = subscribers
            .iter()
            .filter(|c| c.subscribed == mode.1)
            .collect();
        let csv = generate_import_csv(&subs)?;

        info!("Uploading {} listmonk subscribers ({})", subs.len(), mode.0);
        upload_subscribers(import, csv, &config.listmonk_domain, creds).await?;
    }

    Ok(())
}

async fn get_square_customers(creds: &Credentials) -> Result<Vec<SquareCustomer>, Box<dyn Error>> {
    let mut cursor = String::new();
    let mut customers = vec![];
    let mut idx: u32 = 0;

    loop {
        info!("Fetching Square customers ({})", idx);
        let mut res = Client::new()
            .get("https://connect.squareup.com/v2/customers")
            .header("Square-Version", SQUARE_VERSION)
            .query(&[("cursor", cursor)])
            .bearer_auth(&creds.square_api_token)
            .send()
            .await?
            .json::<SquareCustomersResponse>()
            .await?;
        info!("Got {} customers", res.customers.len());
        customers.append(&mut res.customers);
        if let Some(new_cursor) = res.cursor {
            cursor = new_cursor;
            idx += 1;
        } else {
            info!("Reached end of customer list");
            break;
        }
    }

    info!("Found {} total Square customers", customers.len());
    Ok(customers)
}

fn generate_import_csv(subscribers: &Vec<&ListmonkSubscriber>) -> Result<String, Box<dyn Error>> {
    let buf = BufWriter::new(Vec::new());
    let mut writer = csv::Writer::from_writer(buf);
    for customer in subscribers {
        writer.serialize(customer)?;
    }

    let bytes = writer.into_inner()?.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

async fn upload_subscribers(
    import: ListmonkImport,
    csv: String,
    domain: &str,
    creds: &Credentials,
) -> Result<Response, Box<dyn Error>> {
    let form = Form::new()
        .text("params", serde_json::to_string(&import)?)
        .part(
            "file",
            Part::bytes(csv.into_bytes())
                .file_name("import.csv")
                .mime_str("text/csv")?,
        );
    Ok(Client::new()
        .post(format!("https://{domain}/api/import/subscribers"))
        .multipart(form)
        .basic_auth(&creds.listmonk_username, Some(&creds.listmonk_password))
        .send()
        .await?
        .error_for_status()
        .with_context(|| "Failed to upload subscribers to listmonk")?)
}
