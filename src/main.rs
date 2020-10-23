use crate::image_searcher::{GoogleImageSeacher, ImageSearcher, RapidApiImageSeacher};
use serde::Deserialize;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::path::Path;
use thiserror::Error;

mod image_searcher;

#[derive(Error, Debug)]
pub enum ImageBotError {
    #[error("The api returned an unexpected value")]
    Api(String),

    #[error("Error sending the request")]
    NetworkIo(#[from] reqwest::Error),

    #[error("Error loading the config file")]
    DiskIo(#[from] std::io::Error),

    #[error("Error parsing the config file")]
    ConfigParse(#[from] toml::de::Error),
}

#[derive(Deserialize)]
struct Config {
    discord_api_key: String,
    image_search_api_key: String,
    google_cx_id: Option<String>,
    use_google_search: bool,
}

impl Config {
    fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ImageBotError> {
        let file = std::fs::read(path)?;
        Ok(toml::from_slice::<Self>(&file)?)
    }
}
struct Handler {
    searcher: Box<dyn ImageSearcher + Send + Sync>,
}

impl Handler {
    fn new<S: ImageSearcher + Send + Sync + 'static>(searcher: S) -> Self {
        Self {
            searcher: Box::new(searcher),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let split_it = msg.content.split("!image");
        // we skip the first element, because it is always BEFORE the first token found
        // a. if not token is found, its equal to the whole string (the split has size 1)
        // b. if ont token is found, we know the first part is not searched anyway
        let query = split_it.skip(1).find(|word| !word.is_empty());

        match query {
            None => {}
            Some(query) => {
                let url = match self.searcher.search(query).await {
                    Err(e) => {
                        println!("Error getting the image URL: {}", e);
                        println!("Query: {}", query);
                        return;
                    }
                    Ok(url) => url,
                };

                if let Err(why) = msg.channel_id.say(&ctx.http, format!("{}", url)).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, _ready: Ready) {}
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let config = Config::try_from_path("./ImageBot.toml").unwrap();

    let handler = match (config.use_google_search, config.google_cx_id) {
        (false, _) => Handler::new(RapidApiImageSeacher::new(config.image_search_api_key)),

        (true, Some(cx_id)) => {
            Handler::new(GoogleImageSeacher::new(config.image_search_api_key, cx_id))
        }

        (true, None) => {
            panic!("You need to specify the google_cx_id if you want to use the google search API!")
        }
    };

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&config.discord_api_key)
        .event_handler(handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
