mod xiv_util;
mod scraper_util;

use stopwatch::{Stopwatch};
use std::{time::Duration, sync::Mutex, sync::Arc};
use tokio::{task, time};
use poise::serenity_prelude as serenity;
use futures::{Stream, StreamExt};
use poise::command;
use crate::serenity::http::Http;
use regex::Regex;
use lazy_static::lazy_static;
use std::cmp;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
struct Data {
    database:sqlx::SqlitePool,
    pf_listings: Mutex<Vec<xiv_util::PFListing>>
}

async fn autocomplete_datacenter(_ctx: Context<'_>, partial: String) -> impl Stream<Item = String> {
    futures::stream::iter(&["Crystal", "Aether", "Primal", "Elemental", "Gaia", "Mana", "Chaos", "Light", "Materia"])
        .filter(move |name| futures::future::ready(name.starts_with(&partial)))
        .map(|name| name.to_string())
}

async fn autocomplete_duty(_ctx: Context<'_>, partial: String) -> impl Stream<Item = String> {
    futures::stream::iter(&["The Weapon's Refrain (Ultimate)", "The Unending Coil of Bahamut (Ultimate)", "The Epic of Alexander (Ultimate)", "Dragonsong's Reprise (Ultimate)"])
        .filter(move |name| futures::future::ready(name.starts_with(&partial)))
        .map(|name| name.to_string())
}

fn get_embed(data_center: String, duty_name: String, listings: Vec<&xiv_util::PFListing>) -> serenity::builder::CreateEmbed {    
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.color(xiv_util::get_color_from_duty(&duty_name));
    embed.title(format!("{} - {}", duty_name, data_center));
    let max_to_take = std::env::var("MAX_LISTINGS_IN_POST").expect("missing MAX_LISTINGS_IN_POST").parse::<i32>().unwrap();
    println!("Taking max {} listings.", max_to_take);
    let not_taken = std::cmp::max(0, (listings.len() as i32) - max_to_take);

    for listing in listings.iter().take(max_to_take as usize) {
        let author = &listing.author;
        let role_icons_str = listing.slots.iter().map(|x| x.get_emoji_string()).collect::<Vec<String>>().join(" ");
        embed.field(author, role_icons_str, true);
        if listing.flags.chars().count() == 0 {
            embed.field("\u{200b}", &listing.description, true);
        } else {
            embed.field(&listing.flags, &listing.description, true);
        }

        // embed.field("\u{200b}", "\u{200b}", true);
        embed.field(format!("<:ffxivstopwatch:987141580869730324> {}", listing.last_updated), format!("<:ffxivhourglass:987141579879878676> {}", listing.expires_in), true);
    }

    if listings.len() == 0 {
        embed.description("No listings at this time.");
        let mut footer = serenity::builder::CreateEmbedFooter::default();
        footer.text("Or, there are listings but people on this data center don't have the Remote Party Finder dalamud plugin.");
        embed.set_footer(footer);
    }

    if not_taken > 0 {
        embed.field("\u{200b}", format!("[{} {} listing{} not shown.](https://xivpf.com/listings)", not_taken, duty_name, if not_taken == 1 {""} else {"s"}), false);
    }

    embed
}

#[derive(Debug)]
#[allow(dead_code)]
#[derive(Default)]
struct MessageRow {
    message_id: String,
    channel_id: String,
    data_center: String,
    guild_id: String,
    duty_name: String,
    allow_statics: Option<i64>
}

fn parse_time_remaining(last_updated: &str) -> i32 {
    let time_remaining_str: String = last_updated.chars().filter(|c| c.is_digit(10)).collect();

    let mut minutes_since_update = time_remaining_str.parse::<i32>().unwrap_or(0);
    if last_updated.contains("hour") {
        minutes_since_update = 60;
    }
    if last_updated.contains("seconds") {
        minutes_since_update = 0;
    }
    minutes_since_update
}

fn filter_listings<'a>(message_row: &MessageRow, pf_listings: &'a Vec<xiv_util::PFListing>) -> Vec<&'a xiv_util::PFListing> {
    lazy_static! {
        // if this is a match, don't show listing
        static ref RE: Regex = Regex::new(&std::env::var("DESCRIPTION_REGEX").unwrap_or(r".{3,32}#[0-9]{4}".to_string())).unwrap();
    }
    let min_minutes_since_update = (std::env::var("MIN_MINUTES_SINCE_UPDATE").unwrap_or("3".to_string())).parse::<i32>().unwrap(); // don't show pf's last updated more than 5 mins ago
    let min_slots = (std::env::var("MIN_SLOTS").unwrap_or("5".to_string())).parse::<usize>().unwrap();
    let message_allows_statics = if message_row.allow_statics.unwrap_or(1) == 1 { true } else { false };

    let data_center = message_row.data_center.to_string();
    let duty_name = message_row.duty_name.to_string();

    let filtered_listings = pf_listings.iter()
        .filter(|x| {
            x.data_center == data_center 
            && x.title == duty_name
        });
    let filtermax = filtered_listings.clone().map(|x| parse_time_remaining(&x.last_updated)).min().unwrap_or(5);
    let max = cmp::min(cmp::max(filtermax, min_minutes_since_update), 15);
    filtered_listings.filter(|x| {
            let minutes_since_update = parse_time_remaining(&x.last_updated);
            let is_static_ad = RE.is_match(&x.description) || x.slots.len() < min_slots;
            
            if is_static_ad {
                println!("{:?} is classified as a static ad. Message allows statics: {}", x, message_allows_statics);
            }

            if minutes_since_update > max {
                println!("{:?} is had too many minutes ({}) pass since last update. (max {})", x, minutes_since_update, max);
            }

            minutes_since_update <= max
            && (message_allows_statics || !is_static_ad)
        }).collect()
}

async fn update_message(message_row_ref: &MessageRow, data: &Data, http: std::sync::Arc<Http>) -> Result<u32, Error> {
    let mut sw0 = Stopwatch::start_new();

    let message_row = message_row_ref.to_owned();
    let message_id_str = message_row.message_id.to_string();
    let message_id = message_id_str.parse::<u64>().expect("Unable to parse channel id");        
    let channel_id = message_row.channel_id.parse::<u64>().expect("Unable to parse channel id");
    let data_center = message_row.data_center.to_string();
    let duty_name = message_row.duty_name.to_string();
    let message_result = http.get_message(channel_id, message_id).await;
    sw0.stop();

    match message_result {
        Ok(mut message) => {
            let mut sw1 = Stopwatch::start_new();
            let embed = {
                let pf_listings = data.pf_listings.lock().unwrap();
                let filtered_listings = filter_listings(message_row, &pf_listings);
                get_embed(data_center, duty_name, filtered_listings)
            };
            sw1.stop();
            let mut sw2 = Stopwatch::start_new();

            let result = message.edit(&http, |m| m.set_embed(embed)).await;
            sw2.stop();


            match result {
                Ok(_a) => { println!("Successfully edited message, sw0 time: {}, sw1 time: {}, sw2 time: {}", sw0.elapsed_ms(), sw1.elapsed_ms(), sw2.elapsed_ms()); }
                Err(e) => { println!("Error editing message: {}.", e); }
            }
        }
        Err(e) => {
            println!("Error getting message: {}. Couldn't find message for data center {} duty {}, so removing from DB.", e, &data_center, &duty_name);
            sqlx::query!("DELETE FROM messages WHERE message_id=?", message_id_str)
                .fetch_all(&data.database)
                .await.expect("Unable to remove that row from DB");
        }
    }
    Ok(1)
}

async fn update_messages_rustfn_aux(data: &Data, http: std::sync::Arc<Http>) -> Result<usize, Error> {
    println!("update_messages_rustfn_aux called.");
    let messages = sqlx::query_as!(MessageRow, "SELECT message_id, channel_id, guild_id, data_center, duty_name, allow_statics FROM messages")
        .fetch_all(&data.database)
        .await
        .unwrap();
    let update_count = messages.len();

    let sw1 = Stopwatch::start_new();

    for message_row in messages {
        update_message(&message_row, data, Arc::clone(&http)).await?;
    }

    println!("Updated {} messages. sw1: {}", update_count, sw1.elapsed_ms());

    Ok(update_count)
}

async fn update_messages_rustfn(framework: Arc<poise::Framework<Data, std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>>>, http: std::sync::Arc<Http>) -> Result<usize, Error> {
    println!("update_messages_rustfn called.");
    update_messages_rustfn_aux(framework.user_data().await, http).await
}

#[command(slash_command, owners_only, hide_in_help)]
async fn update_messages(ctx: Context<'_>) -> Result<(), Error> {
    println!("update_messages called.");
    let initial_message = ctx.say("Updating messages...").await;
    let mut sw = Stopwatch::start_new();
    let update_count = update_messages_rustfn_aux(&ctx.data(), Arc::clone(&ctx.discord().http)).await?;
    sw.stop();
    initial_message?.edit(ctx, |x| x.content(format!("Updated {} messages. Async elapsed time: {}", update_count, sw.elapsed_ms()))).await.expect("update_messages Couldn't update intial message");
    Ok(())
}


/// Displays FFXIV party finder listings in a discord message. Updates every 5 minutes.
#[poise::command(slash_command, required_permissions = "KICK_MEMBERS")]
async fn display_xivpfs(
    ctx: Context<'_>,
    #[description = "Channel"] channel: serenity::Channel,
    #[description = "Datacenter"] #[autocomplete = "autocomplete_datacenter"] data_center: String,
    #[description = "Duty"] #[autocomplete = "autocomplete_duty"] duty_name: String,
    #[description = "Allow Statics"] allow_statics: bool,
) -> Result<(), Error> {
    let initial_message = ctx.say(format!("Adding PF listings display...")).await;
    let author_name = &ctx.author().name.to_string();
    println!("display_xivpfs called, author: {}", author_name);

    let response = match channel.guild() {
        Some(guild_channel) => {
            if guild_channel.kind.name() != "text" {
                format!("Can't post message, {} is not a text channel.", guild_channel.name())
            } else {
                let guild_name = ctx.guild().unwrap().name;
                let guild_id = ctx.guild_id().unwrap().0.to_string();
                println!("display_xivpfs player name: {}, duty_name: {}, guild name: {}", guild_name, duty_name, author_name);
                let allow_statics_i = if allow_statics {1} else {0};
                sqlx::query!("INSERT OR IGNORE INTO guilds(guild_id, guild_name) VALUES(?, ?)", guild_id, guild_name)
                    .fetch_all(&ctx.data().database)
                    .await
                    .unwrap();
                

                    let embed = {
                    let pf_listings = ctx.data().pf_listings.lock().unwrap();
                    let filtered_listings = filter_listings(&MessageRow { data_center: data_center.to_string(), duty_name: duty_name.to_string(), allow_statics: Some(allow_statics_i), ..MessageRow::default() }, &pf_listings);
                    get_embed(data_center.to_string(), duty_name.to_string(), filtered_listings)
                };
                let channel_id = guild_channel.id;
                let message = channel_id.send_message(&ctx.discord().http, |m| m.set_embed(embed)).await.expect("something");
                let message_id = message.id.0.to_string();
                let channel_id_str = channel_id.0.to_string();
                let guild_id = ctx.guild_id().unwrap().0.to_string();
                sqlx::query!("INSERT INTO messages(message_id, channel_id, guild_id, data_center, duty_name, allow_statics) VALUES(?, ?, ?, ?, ?, ?)", message_id, channel_id_str, guild_id, data_center, duty_name, allow_statics_i)
                    .fetch_all(&ctx.data().database)
                    .await
                    .unwrap();
                format!("Created updating message in channel {}", guild_channel.name())
            }
        }
        None => {
            format!("Not a valid channel.")
        }
    };

    initial_message.unwrap().edit(ctx, |x| x.content(response)).await.expect("display_xivpfs Couldn't edit initial message.");
    Ok(())
}

#[poise::command(owners_only, prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn update_xivpfs_rustfn_aux(data: &Data) -> Result<(), Error> {
    println!("update_xivpfs_rustfn_aux called.");
    let html = reqwest::get("https://xivpf.com/listings")
        .await?
        .text()
        .await?;

    let listings = scraper_util::get_listings(html);
    *data.pf_listings.lock().unwrap() = listings;
    Ok(())
}


async fn update_xivpfs_rustfn(framework: Arc<poise::Framework<Data, std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>>>) -> Result<(), Error> {
    println!("update_xivpfs_rustfn called.");
    update_xivpfs_rustfn_aux(framework.user_data().await).await?;
    Ok(())
}

#[poise::command(slash_command, owners_only, hide_in_help)]
async fn update_xivpfs(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!("Updating xivpfs...")).await?;
    update_xivpfs_rustfn_aux(&ctx.data()).await?;
    Ok(())
}

async fn init_bot() {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    //sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

    let pf_listings = Mutex::new(scraper_util::get_sample_listings().await);

    let bot = Data {
        database,
        pf_listings    
    };

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let token_2 = token.to_string();

    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![display_xivpfs(), register()], //update_messages(), update_xivpfs(), update_message_sync()
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(bot) 
        })).build().await.expect("Unable to initialize framework");


    let cloned = Arc::clone(&framework);


    task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(5*60*1000));
        let http = Arc::new(serenity::http::Http::new(&token_2));

        loop {
            interval.tick().await;
            println!("Update ticked.");
            update_xivpfs_rustfn(Arc::clone(&framework)).await.expect("Couldn't update_xivpfs_rustfn");
            update_messages_rustfn(Arc::clone(&framework), Arc::clone(&http)).await.expect("Couldn't update_messages_rustfn");
        }
    });

    poise::Framework::start(cloned).await.expect("Unable to start poise framework.");
}


#[tokio::main]
async fn main() {
    init_bot().await;
}