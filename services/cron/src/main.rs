use sqlx::{PgPool, Postgres, Pool};

use poise::serenity_prelude::{self as serenity, EditMessage};

// Temporary
pub async fn create_deposit_embed_message(conn: &Pool<Postgres>) -> Result<poise::CreateReply, anyhow::Error> {
    let rows = sqlx::query!("SELECT DISTINCT website_id FROM deposit").fetch_all(conn).await?;
    let rows_withdraw = sqlx::query!("SELECT DISTINCT website_id FROM withdraw").fetch_all(conn).await?;
    let get_disabled = |website_id: &str| {
        rows.iter().any(|r| r.website_id == website_id)
    };
    let get_disabled_withdraw = |website_id: &str| {
        rows_withdraw.iter().any(|r| r.website_id == website_id)
    };
    let get_style = |website_id: &str| {
        if get_disabled(website_id) {
            poise::serenity_prelude::ButtonStyle::Secondary
        } else {
            poise::serenity_prelude::ButtonStyle::Primary
        }
    };
    let get_style_withdraw = |website_id: &str| {
        if get_disabled_withdraw(website_id) {
            poise::serenity_prelude::ButtonStyle::Secondary
        } else {
            poise::serenity_prelude::ButtonStyle::Danger
        }
    };
    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("deposit-init-1")
            .label("Website 1")
            .disabled(get_disabled("1"))
            .style(get_style("1")),
        serenity::CreateButton::new("deposit-init-2")
            .label("Website 2")
            .disabled(get_disabled("2"))
            .style(get_style("2")),
        serenity::CreateButton::new("deposit-init-3")
            .label("Website 3")
            .disabled(get_disabled("3"))
            .style(get_style("3")),
        serenity::CreateButton::new("deposit-init-4")
            .label("Website 4")
            .disabled(get_disabled("4"))
            .style(get_style("4")),
        serenity::CreateButton::new("deposit-init-5")
            .label("Website 5")
            .disabled(get_disabled("5"))
            .style(get_style("5")),
    ]),
    serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("withdraw-init-1")
            .label("Website 1")
            .disabled(get_disabled("1"))
            .style(get_style_withdraw("1")),
        serenity::CreateButton::new("withdraw-init-2")
            .label("Website 2")
            .disabled(get_disabled("2"))
            .style(get_style_withdraw("2")),
        serenity::CreateButton::new("withdraw-init-3")
            .label("Website 3")
            .disabled(get_disabled("3"))
            .style(get_style_withdraw("3")),
        serenity::CreateButton::new("withdraw-init-4")
            .label("Website 4")
            .disabled(get_disabled("4"))
            .style(get_style_withdraw("4")),
        serenity::CreateButton::new("withdraw-init-5")
            .label("Website 5")
            .disabled(get_disabled("5"))
            .style(get_style_withdraw("5")),
    ])];

    Ok(poise::CreateReply::default()
            .content("Deposit process explanation")
            .components(components))
}

pub async fn get_deposit_edit_message(conn: &Pool<Postgres>) -> Result<EditMessage, anyhow::Error> {
    let builder = create_deposit_embed_message(conn).await?;
    Ok(EditMessage::default().content("Deposit process explanation").components(builder.components.unwrap()))
}

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

    loop {
        let mut affected_rows = 0;

        // Cleans deposit table of process older than the 5 minutes timeout
        let rows = sqlx::query!("SELECT website_id, interaction_token FROM deposit WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").fetch_all(&conn).await?;
        // let query = sqlx::query!("DELETE FROM deposit WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").execute(&conn).await?;
        let query = sqlx::query!("UPDATE deposit SET is_check=TRUE WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").execute(&conn).await?;
        for r in rows.iter() {
            let data = serenity::CreateInteractionResponseFollowup::new().ephemeral(true)
                .content(format!("Your deposit process on website {} was marked as finished after 5 minutes", r.website_id));
            client.http.create_followup_message(&r.interaction_token, &data, vec![]).await?;
            if let Ok(Some(embed_id)) = sqlx::query!("SELECT message_id,channel_id FROM embed").fetch_optional(&conn).await {
                let message_id = serenity::MessageId::new(embed_id.message_id as u64);
                let channel_id = serenity::ChannelId::new(embed_id.channel_id as u64);
                if let Ok(mut msg) = client.http.get_message(channel_id, message_id).await {
                    if let Ok(builder) = get_deposit_edit_message(&conn).await {
                        let _ = msg.edit(&client.http, builder).await;
                    }
                }
            };
        }
        affected_rows += query.rows_affected();

        // Cleans withdraw table of process older than the 5 minutes timeout
        let rows = sqlx::query!("SELECT website_id, interaction_token FROM withdraw WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").fetch_all(&conn).await?;
        // let query = sqlx::query!("DELETE FROM withdraw WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").execute(&conn).await?;
        let query = sqlx::query!("UPDATE withdraw SET is_check=TRUE WHERE start_date < CURRENT_TIMESTAMP - INTERVAL '5 minutes' AND is_check=false").execute(&conn).await?;
        for r in rows.iter() {
            let data = serenity::CreateInteractionResponseFollowup::new().ephemeral(true)
                .content(format!("Your withdraw process on website {} was marked as finished after 5 minutes", r.website_id));
                client.http.create_followup_message(&r.interaction_token, &data, vec![]).await?;
                if let Ok(Some(embed_id)) = sqlx::query!("SELECT message_id,channel_id FROM embed").fetch_optional(&conn).await {
                    let message_id = serenity::MessageId::new(embed_id.message_id as u64);
                    let channel_id = serenity::ChannelId::new(embed_id.channel_id as u64);
                    if let Ok(mut msg) = client.http.get_message(channel_id, message_id).await {
                        if let Ok(builder) = get_deposit_edit_message(&conn).await {
                            let _ = msg.edit(&client.http, builder).await;
                        }
                    }
                };
        }
        affected_rows += query.rows_affected();

        println!("Deleted {} records", affected_rows);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}