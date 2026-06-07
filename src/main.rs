use clap::{Parser, ValueEnum};
use serenity::{Client, all::GatewayIntents};

mod bot;
use bot::Handler;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum Action {
    /// Kick violators.
    Kick,
    /// Ban violators.
    Ban,
    /// Tattle on violators.
    LogOnly,
}

#[derive(Parser, Debug)]
struct Config {
    /// Token for the Discord bot per the "Bot" page on the Applications screen of the developer portal.
    #[arg(long, env = "DISCORD_TOKEN")]
    bot_token: String,

    /// Comma-separated list of channel IDs (snowflakes) to monitor & enforce in.
    #[arg(long, env = "WATCHLIST_CHANNEL_IDS", value_delimiter = ',')]
    channel_watchlist: Vec<String>,
    /// Comma-separated list of user IDs (snowflakes) to ignore messages from.
    #[arg(long, env = "WHITELIST_USER_IDS", value_delimiter = ',')]
    user_whitelist: Vec<String>,
    /// Comma-separated list of role IDs (snowflakes) to ignore messages from.
    #[arg(long, env = "WHITELIST_ROLE_IDS", value_delimiter = ',')]
    role_whitelist: Vec<String>,

    /// Channel ID (snowflake) to send a message in upon enforcement.
    #[arg(long, env = "LOGGING_CHANNEL_ID")]
    logging_channel: String,

    /// Role ID (snowflake) to ping in "log only" mode, messages from these users will also be ignored.
    #[arg(long, env = "ADMIN_ROLE_ID")]
    admin_role: String,
    /// Action to take upon violation.
    #[arg(long, env = "VIOLATION_ACTION")]
    action: Action,
    /// Number of days worth of messages to automatically delete upon violation when the violation action is to "ban".
    #[arg(long, env = "BAN_DELETE_MESSAGES_X_DAYS_PRIOR")]
    dmd: u8,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    let mut client = Client::builder(&config.bot_token, GatewayIntents::GUILD_MESSAGES)
        .event_handler(Handler::from_config(&config))
        .await
        .expect("Err creating client");

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}");
    }
}
