use std::sync::Arc;

use poise::serenity_prelude::{self as serenity, ChannelId, CreateMessage, GuildId};
use sqlx::{Pool, Postgres};

pub async fn send_log_to_discord(http: &Arc<serenity::Http>, conn: &Pool<Postgres>, guild_id: GuildId, msg: &str) -> anyhow::Result<()>{
    let guild_id: i64 = guild_id.into();
    let Some(row) = sqlx::query!("SELECT channel_id FROM log WHERE guild_id=$1", guild_id).fetch_optional(conn).await? else {
        return Ok(());
    };
    let channel_id = ChannelId::new(row.channel_id as u64);
    let _ = channel_id.send_message(http, CreateMessage::default().content(msg)).await;
    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        // Loads dotenv file
        let _ = dotenv::dotenv();

        let app_id = serenity::ApplicationId::new(std::env::var("DISCORD_APP_ID").expect("missing DISCORD_APP_ID").parse()?);
        let client = serenity::Client::builder(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), serenity::GatewayIntents::non_privileged()).application_id(app_id).await.expect("Err creating client");    

        let database_url = std::env::var("DATABASE_URL").expect("Expected a database url in the environment");
        let conn = sqlx::PgPool::connect(&database_url).await?;

        // let rt = tokio::runtime::Handle::current();

        // let logger = DiscordLogger { http: client.http };

        // let decorator = slog_term::TermDecorator::new().build();
        // let drain = slog_term::FullFormat::new(decorator).build().fuse();
        // let drain = slog_async::Async::new(drain).build().fuse();
        // let log = slog::Logger::root(drain, o!());

        // slog_scope::set_global_logger(logger);
        // slog_stdlog::init().unwrap();
        // slog_stdlog::set_logger(log.clone()).unwrap();

        // let logger = Box::new(DiscordLogger { http: client.http, rt });

        // log::set_max_level(LevelFilter::Info);
        // log::set_boxed_logger(logger).expect("Failed to set logger");

        // info!("{}: This is an info message.", 98989789789784_i64);
        // error!("{}: This is an error message.", 98989789789784_i64);

        let guild_id = std::env::var("GUILD_ID")?;
        send_log_to_discord(&client.http, &conn, GuildId::new(guild_id.parse()?), "bababoey").await?;
        assert_eq!(7, 6);
        Ok(())
    }
}
