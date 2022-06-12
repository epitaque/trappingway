mod xiv_util;
mod scraper_util;

use std::str::FromStr;
use std::time::Duration;
use tokio::{task, time};
use poise::serenity_prelude as serenity;
use futures::{Stream, StreamExt};
use crate::serenity::{CreateEmbedFooter, CreateEmbedAuthor};
use crate::serenity::ChannelId;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {
    database: sqlx::SqlitePool,
}

async fn autocomplete_datacenter(_ctx: Context<'_>, partial: String) -> impl Stream<Item = String> {
    futures::stream::iter(&["Crystal", "Aether", "Primal", "Elemental", "Gaia", "Mana", "Chaos", "Light", "Materia"])
        .filter(move |name| futures::future::ready(name.starts_with(&partial)))
        .map(|name| name.to_string())
}

async fn autocomplete_duty(_ctx: Context<'_>, partial: String) -> impl Stream<Item = String> {
    futures::stream::iter(&["The Weapon's Refrain (Ultimate)", "The Unending Coil of Bahamut (Ultimate)", "The Weapon's Refrain (Ultimate)", "Dragonsong's Reprise (Ultimate)", "The Epic of Alexander (Ultimate)"])
        .filter(move |name| futures::future::ready(name.starts_with(&partial)))
        .map(|name| name.to_string())
}

fn get_embed(data_center: String, duty_name: String, listings: Vec<&xiv_util::PFListing>) -> serenity::builder::CreateEmbed {
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.color(1u32);
    embed.title(format!("{} - {}", duty_name, data_center));
    for listing in listings.iter().take(15) {
        if listing.slots.len() == 0 || listing.slots.iter().any(|x| x.available_jobs.len() == 0) {
            continue
        }

        // let author = listing.author.split("@").next().unwrap().trim_end();
        let author = &listing.author;
        let role_icons_str = listing.slots.iter().map(|x| x.get_emoji_string()).collect::<Vec<String>>().join(" ");

        // let field_title = format!("{} - {}", snipped_author, listing.flags);
        // let field_description = format!("{} {}", role_icons_str, listing.description);

        embed.field(author, role_icons_str, true);
        embed.field(&listing.flags, &listing.description, true);
        embed.field("\u{200b}", '\u{200b}', true);
    }
    // embed.description("Embed description");
    // let mut footer = CreateEmbedFooter::default();
    // footer.url("https://github.com/epitaque/raidingway");
    // footer.text("Source - Powered by xivpfs");

    // let mut author = CreateEmbedAuthor::default();
    // author.url("https://github.com/epitaque/raidingway");
    // author.name("Source - Powered by xivpfs");

    // embed.set_author(author);
    // embed.set_footer(footer);
    // let message = channel_id.send_message(&ctx.discord().http, |m| m.set_embed(embed)).await.expect("something");
    // let message_id = message.id.0.to_string();
    // let channel_id_str = channel_id.0.to_string();
    // let guild_id = ctx.guild_id().unwrap().0.to_string();
    // sqlx::query!("INSERT INTO messages(message_id, channel_id, guild_id, data_center, duty_name) VALUES(?, ?, ?, ?, ?)", message_id, channel_id_str, guild_id, data_center, duty)
    //     .fetch_all(&ctx.data().database)
    //     .await
    //     .unwrap();

    // println!("Success, message id: {}", message.id.0.to_string());
    embed
}

#[poise::command(slash_command, prefix_command)]
async fn update_xivpfs(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("Fetching pfs and updating posts.");
    ctx.say(response);

    let messages = sqlx::query!("SELECT message_id, channel_id, guild_id, data_center, duty_name FROM messages")
    .fetch_all(&ctx.data().database)
    .await
    .unwrap();
    let listings = scraper_util::get_sample_listings().await;

    let message_len = messages.len();
    for message_row in messages {
        let message_id = message_row.message_id.unwrap().parse::<u64>().expect("Unable to parse channel id");
        let channel_id = message_row.channel_id.parse::<u64>().expect("Unable to parse channel id");

        let data_center = message_row.data_center;
        let duty_name = message_row.duty_name;

        let mut message = ctx.discord().http.get_message(channel_id, message_id).await.expect("Unable to get message from message id and channel");
        let filtered_listings = listings.iter().filter(|x| x.data_center == data_center && x.title == duty_name).collect();
        let embed = get_embed(data_center, duty_name, filtered_listings);
        let result = message.edit(&ctx.discord().http, |m| m.set_embed(embed)).await;
        match result {
            Ok(_a) => { println!("Successfully edited message."); }
            Err(e) => { println!("Error editting message: {}.", e); }
        }
        println!("Found message: {}", message_id.to_string());
    }

    Ok(())
}


/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn display_xivpfs(
    ctx: Context<'_>,
    #[description = "Channel"] channel: serenity::Channel,
    #[description = "Datacenter"] #[autocomplete = "autocomplete_datacenter"] data_center: String,
    #[description = "Duty"] #[autocomplete = "autocomplete_duty"] duty_name: String,
) -> Result<(), Error> {
    
    let response = match channel.guild() {
        Some(guild_channel) => {
            if guild_channel.kind.name() != "text" {
                format!("Can't post message, {} is not a text channel.", guild_channel.name())
            } else {
                // guild_channel.send_message(cache_http: impl CacheHttp, f: F)
                // guild_channel.say(ctx.discord().http.as_ref(), get_message(datacenter).await).await?;
                let guild_name = ctx.guild().unwrap().name;
                let guild_id = ctx.guild_id().unwrap().0.to_string();
                sqlx::query!("INSERT OR IGNORE INTO guilds(guild_id, guild_name) VALUES(?, ?)", guild_id, guild_name)
                    .fetch_all(&ctx.data().database)
                    .await
                    .unwrap();
                
                // let response2 = format!("Add guild result: {}:\n", guilds.len());

                let mut listings = scraper_util::get_sample_listings().await;
                listings = listings.drain(0..).filter(|x| x.data_center == data_center && x.title == duty_name).collect();

                // guild_channel.id.say(&ctx.discord().http, response2).await.expect("Error sending discord message");
                let embed = get_embed(data_center.to_string(), duty_name.to_string(), listings).await;
                let channel_id = guild_channel.id;
                let message = channel_id.send_message(&ctx.discord().http, |m| m.set_embed(embed)).await.expect("something");
                let message_id = message.id.0.to_string();
                let channel_id_str = channel_id.0.to_string();
                let guild_id = ctx.guild_id().unwrap().0.to_string();
                sqlx::query!("INSERT INTO messages(message_id, channel_id, guild_id, data_center, duty_name) VALUES(?, ?, ?, ?, ?)", message_id, channel_id_str, guild_id, data_center, duty_name)
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
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
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

    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

    let bot = Data {
        database,
    };


    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![display_xivpfs(), register(), update_xivpfs()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {

            Ok(bot) 
        }));

    framework.run().await.unwrap();
}

#[tokio::main]
async fn main() {
    init_bot().await;
    // let mut listings = scraper_util::get_sample_listings().await;
    // let molisting = listings.drain(0..).filter(|x| x.author == "Xepha Sgiathan @ Malboro").collect::<Vec<xiv_util::PFListing>>();
    // println!("{:#?}", molisting);

    // for job in &molisting.first().unwrap().slots[7].available_jobs {
    //     println!("Job {:#?}", job);
    //     println!("Role {:#?}", job.get_role());
    // }
}

// #[tokio::main]
// async fn main() {
//     // let html = reqwest::get("https://bloomberg.com/")
//     //     .await?
//     //     .text()
//     //     .await?;
//     let html = fs::read_to_string("scrape_example.html").expect("Unable to read");
//     let listings = get_listings(html);
//     println!("A listing: {}", listings[0].to_string());



//     // let forever = task::spawn(async {
//     //     let mut interval = time::interval(Duration::from_millis(1000));

//     //     loop {
//     //         interval.tick().await;
//     //         do_something().await;
//     //     }
//     // });

//     // forever.await;
// }
