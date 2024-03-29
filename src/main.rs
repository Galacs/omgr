use core::fmt;

use poise::{serenity_prelude as serenity, CreateReply};

use poise::serenity_prelude::{UserId, CreateEmbed};
use sqlx::{Postgres, Pool, PgPool};

#[derive(Debug)]
pub struct Data(Pool<Postgres>, roboat::Client);
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod embeds;
mod event;

/// Ping command
#[poise::command(slash_command, prefix_command)]
async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Sets global withdraw tax
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
async fn set_tax(
    ctx: Context<'_>,
    #[description = "Website"] website_id: WebsiteId,
    #[description = "Tax in %"] rate: f32,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    sqlx::query!("UPDATE tax SET rate = $1 WHERE tax='withdraw' AND website_id=$2", rate, website_id.to_string()).execute(conn).await?;
    ctx.say(format!("Tax is now set to {}% for withdrawals for website {}", rate, website_id)).await?;
    Ok(())
}

/// Sets the guild's log channel
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
async fn set_log(
    ctx: Context<'_>,
    #[description = "Log channel id"] channel: serenity::Channel,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let guild_id: i64 = ctx.guild_id().ok_or("in pm")?.into();
    let channel_id: i64 = channel.id().into();
    sqlx::query!("INSERT INTO log(guild_id,channel_id) VALUES ($1,$2) ON CONFLICT(guild_id) DO UPDATE SET channel_id=$2", guild_id, channel_id).execute(conn).await?;
    ctx.say(format!("The log channel is now {}", channel)).await?;
    Ok(())
}
#[derive(poise::ChoiceParameter)]
enum WebsiteId {
    Website_1,
    Website_2,
    Website_3,
    Website_4,
    Website_5,
    Website_6,
    Website_7,
}

impl fmt::Display for WebsiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebsiteId::Website_1 => write!(f, "1"),
            WebsiteId::Website_2 => write!(f, "2"),
            WebsiteId::Website_3 => write!(f, "3"),
            WebsiteId::Website_4 => write!(f, "4"),
            WebsiteId::Website_5 => write!(f, "5"),
            WebsiteId::Website_6 => write!(f, "6"),
            WebsiteId::Website_7 => write!(f, "7"),
        }
    }
}

/// Sets the specified's website stock
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
async fn set_stock(
    ctx: Context<'_>,
    #[description = "Website ID"] website: WebsiteId,
    #[description = "stock"] stock: i32,
) -> Result<(), Error> {
    let conn = &ctx.data().0;

    sqlx::query!("UPDATE stock SET stock = $1 WHERE website_id=$2", stock, website.to_string()).execute(conn).await?;
    ctx.send(CreateReply::default().ephemeral(true).content(format!("The stock is now set to {} on website {}", stock, website))).await?;
    Ok(())
}

/// Get all stocks
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
async fn get_stock(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;

    let rows = sqlx::query!("SELECT * FROM stock").fetch_all(conn).await?;
    let map: std::collections::HashMap<_, _> = rows.iter().map(|x| (x.website_id.clone(), x.stock)).collect();
    ctx.send(CreateReply::default().embed(CreateEmbed::default().title("Stock")
        .field("Website 1", map.get("1").unwrap_or(&0).to_string(), false)
        .field("Website 2", map.get("2").unwrap_or(&0).to_string(), false)
        .field("Website 3", map.get("3").unwrap_or(&0).to_string(), false)
        .field("Website 4", map.get("4").unwrap_or(&0).to_string(), false)
        .field("Website 5", map.get("5").unwrap_or(&0).to_string(), false)
        .field("Website 6", map.get("6").unwrap_or(&0).to_string(), false)
        .field("Website 7", map.get("7").unwrap_or(&0).to_string(), false)
    )).await?;
    Ok(())
}

pub async fn create_user_balance(user_id: i64, conn: &Pool<Postgres>) -> Result<(), Error> {
    sqlx::query!("INSERT INTO balances(discord_id) VALUES ($1)", user_id).execute(conn).await?;
    Ok(())
}

// Returns true if user wasn't in the db
pub async fn exists_or_create_user(user_id: i64, conn: &Pool<Postgres>) -> Result<bool, Error> {
    let mut bool = false;
    if sqlx::query!("SELECT * FROM balances WHERE discord_id = $1", user_id).fetch_one(conn).await.is_err() {
        create_user_balance(user_id, conn).await?;
        bool = true;
    };
    Ok(bool)
}

/// Displays your or another user's account balance
#[poise::command(slash_command, prefix_command)]
async fn balance(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id: i64 = ctx.author().id.into();
    
    exists_or_create_user(user_id, conn).await?;

    let balance = sqlx::query!("SELECT balance FROM balances WHERE discord_id=$1", user_id).fetch_one(conn).await?.balance;

    match &user {
        Some(u) => ctx.say(format!("{} has {}", u, balance)).await?,
        None => ctx.say(format!("You have {}", balance)).await?,
    };

    Ok(())
}

#[poise::command(slash_command, prefix_command, owners_only)]
async fn add_balance(
    ctx: Context<'_>,
    #[description = "Amount"] amount: i64,
    #[description = "User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = user.unwrap_or(ctx.author().clone());
    let user_id: i64 = user.id.into();
    sqlx::query!("UPDATE balances SET balance = balance + $2 WHERE discord_id = $1", user_id, amount).execute(conn).await?;
    ctx.say(format!("Added {} to <@{}>'s balance", amount, user_id)).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Loads dotenv file
    let _ = dotenv::dotenv();

    // DB
    let database_url = std::env::var("DATABASE_URL").expect("Expected a database url in the environment");
    let conn = PgPool::connect(&database_url).await?;
    sqlx::migrate!().run(&conn).await?;

    // Roblox API client
    let client = roboat::ClientBuilder::new()
        .roblosecurity(std::env::var("ROBLOSECURITY").unwrap_or_default())
        .build();

    let owner_id = {
        let env_var = std::env::var("OWNER_ID");
        if let Ok(str) = env_var {
            UserId::new(str.parse().unwrap_or_default())
        } else {
            UserId::default()
        }
    };
    let mut owners = std::collections::HashSet::<serenity::UserId>::new();
    owners.insert(owner_id);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), set_tax(), set_log(), get_stock(), set_stock(), balance(), add_balance(), embeds::create_deposit_embed(), embeds::update_deposit_embed()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
            },
            owners,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Ok(var) = std::env::var("GUILD_ID") {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(var.parse().expect("GUILD_ID should be an integer"))).await?;
                }
                else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(Data(conn, client))
            })
        }).build();

    let client = serenity::ClientBuilder::new(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
    Ok(())
}