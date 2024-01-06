use poise::serenity_prelude as serenity;

use poise::serenity_prelude::UserId;
use sqlx::{Postgres, Pool, PgPool};

#[derive(Debug)]
pub struct Data(Pool<Postgres>);
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
    #[description = "Tax in %"] rate: f32,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    sqlx::query!("UPDATE tax SET rate = $1 WHERE tax='withdraw'", rate).execute(conn).await?;
    ctx.say(format!("Tax is now set to {}% for withdrawals", rate)).await?;
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

    let owner_id = {
        let env_var = std::env::var("OWNER_ID");
        if let Ok(str) = env_var {
            UserId::new(str.parse().unwrap_or_default())
        } else {
            UserId::new(0)           
        }
    };
    let mut owners = std::collections::HashSet::<serenity::UserId>::new();
    owners.insert(owner_id);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), set_tax(), embeds::create_deposit_embed(), embeds::update_deposit_embed()],
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
                Ok(Data(conn))
            })
        }).build();

    let client = serenity::ClientBuilder::new(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
    Ok(())
}