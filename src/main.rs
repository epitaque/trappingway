mod xiv_util;
mod scraper_util;

use std::{time::Duration, sync::Mutex};
use tokio::{task, time};
use poise::serenity_prelude as serenity;
use futures::{Stream, StreamExt};
use poise::command;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {
    database: sqlx::SqlitePool,
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
    for listing in listings.iter().take(15) {
        if listing.slots.len() == 0 || listing.slots.iter().any(|x| x.available_jobs.len() == 0) {
            continue
        }

        let author = &listing.author;
        let role_icons_str = listing.slots.iter().map(|x| x.get_emoji_string()).collect::<Vec<String>>().join(" ");
        embed.field(author, role_icons_str, true);
        if listing.flags.chars().count() == 0 {
            embed.field("\u{200b}", &listing.description, true);
        } else {
            embed.field(&listing.flags, &listing.description, true);
        }

        embed.field("\u{200b}", '\u{200b}', true);
    }

    embed
}

#[command(slash_command, owners_only, hide_in_help)]
async fn update_messages(ctx: Context<'_>) -> Result<(), Error> {
    println!("update_messages called.");
    let initial_message = ctx.say("Updatings messages...").await;

    let messages = sqlx::query!("SELECT message_id, channel_id, guild_id, data_center, duty_name FROM messages")
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();
    let mut update_count = 0;

    for message_row in messages {
        let message_id_str = message_row.message_id.unwrap();
        let message_id = message_id_str.parse::<u64>().expect("Unable to parse channel id");
        let channel_id = message_row.channel_id.parse::<u64>().expect("Unable to parse channel id");

        let data_center = message_row.data_center;
        let duty_name = message_row.duty_name;

        let message_result = ctx.discord().http.get_message(channel_id, message_id).await;

        match message_result {
            Ok(mut message) => {
                let embed = {
                    let pf_listings = ctx.data().pf_listings.lock().unwrap();
                    let filtered_listings = pf_listings.iter().filter(|x| x.data_center == data_center && x.title == duty_name).collect();
                    get_embed(data_center, duty_name, filtered_listings)
                };
                let result = message.edit(&ctx.discord().http, |m| m.set_embed(embed)).await;
                match result {
                    Ok(_a) => { println!("Successfully edited message."); }
                    Err(e) => { println!("Error editing message: {}.", e); }
                }
                update_count = update_count + 1;
                ()
            }
            Err(e) => {
                println!("Couldn't find message for data center {} duty {}, so removing from DB.", &data_center, &duty_name);
                sqlx::query!("DELETE FROM messages WHERE message_id=?", message_id_str)
                    .fetch_all(&ctx.data().database)
                    .await;
                ()
            }
        }

    }

    initial_message.unwrap().edit(ctx, |x| x.content(format!("Updated {} messages.", update_count))).await;
    Ok(())
}


/// Displays your or another user's account creation date
#[poise::command(slash_command, owners_only)]
async fn display_xivpfs(
    ctx: Context<'_>,
    #[description = "Channel"] channel: serenity::Channel,
    #[description = "Datacenter"] #[autocomplete = "autocomplete_datacenter"] data_center: String,
    #[description = "Duty"] #[autocomplete = "autocomplete_duty"] duty_name: String,
) -> Result<(), Error> {
    let initial_message = ctx.say(format!("Adding PF listings display...")).await;

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
                let embed = {
                    let pf_listings = ctx.data().pf_listings.lock().unwrap();
                    let filtered_listings = pf_listings.iter().filter(|x| x.data_center == data_center && x.title == duty_name).collect::<Vec<_>>();
                    get_embed(data_center.to_string(), duty_name.to_string(), filtered_listings)
                };
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

    initial_message.unwrap().edit(ctx, |x| x.content(response)).await;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(slash_command, owners_only, hide_in_help)]
async fn update_xivpfs(ctx: Context<'_>) -> Result<(), Error> {
    let initial_message = ctx.say(format!("Updating xivpfs...")).await;
    println!("Update xivpfs called.");
    let html = reqwest::get("https://xivpf.com/listings")
        .await?
        .text()
        .await?;


    let listings = scraper_util::get_listings(html);
    
    *ctx.data().pf_listings.lock().unwrap() = listings;
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

    let pf_listings = Mutex::new(scraper_util::get_sample_listings().await);

    let bot = Data {
        database,
        pf_listings
    };


    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![display_xivpfs(), register(), update_messages(), update_xivpfs()],
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

    task::spawn(async {
        let mut interval = time::interval(Duration::from_millis(5*60*1000));

        loop {
            interval.tick().await;
            println!("Update ticked.");
            update_xivpfs();
            update_messages();
        }
    });


    init_bot().await;

    // let mut delay = time::interval(Duration::from_millis(5000));
    // delay.tick().await;
    // update_xivpfs();

    // let forever = task::spawn(async {
    //     let mut interval = time::interval(Duration::from_millis(60000));

    //     loop {
    //         interval.tick().await;
    //         update_xivpfs();
    //     }
    // });

    // forever.await;

    // let mut listings = scraper_util::get_sample_listings().await;
    // let filtered_listings = listings.iter().filter(|x| x.data_center == "Crystal" && x.title == "Dragonsong's Reprise (Ultimate)").collect::<Vec<_>>();


    // let forever = task::spawn(async {
    //     let mut interval = time::interval(Duration::from_millis(1000));

    //     loop {
    //         interval.tick().await;
    //         do_something().await;
    //     }
    // });

    // forever.await;


    // let molisting = listings.drain(0..).filter(|x| x.author == "Karna Masta @ Brynhildr").collect::<Vec<xiv_util::PFListing>>();
    // println!("{:#?}", filtered_listings);

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
