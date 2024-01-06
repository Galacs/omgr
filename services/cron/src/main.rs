use sqlx::PgPool;

use poise::serenity_prelude as serenity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Loads dotenv file
    let _ = dotenv::dotenv();

    // DB
    let database_url = std::env::var("DATABASE_URL").expect("Expected a database url in the environment");
    let conn = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&conn).await?;

    // Discord api clint
    let app_id = serenity::ApplicationId::new(std::env::var("DISCORD_APP_ID").expect("missing DISCORD_APP_ID").parse()?);
    let client = serenity::Client::builder(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), serenity::GatewayIntents::non_privileged()).application_id(app_id).await.expect("Err creating client");

    // Cleans deposit table of process older than the 5 minutes timeout
    let rows = sqlx::query!("SELECT website_id, interaction_token FROM deposit WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes'").fetch_all(&conn).await?;
    let query = sqlx::query!("DELETE FROM deposit WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes'").execute(&conn).await?;
    for r in rows.iter() {
        let data = serenity::CreateInteractionResponseFollowup::new().ephemeral(true)
            .content(format!("You deposit process on website {} was cancelled after 5 minutes", r.website_id));
        client.http.create_followup_message(&r.interaction_token, &data, vec![]).await?;
    }

    println!("Deleted {} records", query.rows_affected());

    Ok(())
}