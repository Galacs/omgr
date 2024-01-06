use poise::serenity_prelude::{self as serenity, FullEvent};

use crate::{Data, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::InteractionCreate { interaction: serenity::Interaction::Component(interaction) } => {
            let conn = &data.0;
            let custom_id = &interaction.data.custom_id;
            let discord_id: i64 = interaction.user.id.into();
            if custom_id.starts_with("deposit") {
                let Some(website_id) = &interaction.data.custom_id.split('-').nth(2) else {
                    return Ok(());
                };
                if custom_id.starts_with("deposit-init") {
                    let reply = {
                        let components = vec![serenity::CreateActionRow::Buttons(vec![
                            serenity::CreateButton::new(format!("deposit-cancel-{}", website_id))
                                .label("Cancel deposit")
                                .style(poise::serenity_prelude::ButtonStyle::Secondary),
                            serenity::CreateButton::new(format!("deposit-finish-{}", website_id))
                                .label("Finish deposit")
                                .style(poise::serenity_prelude::ButtonStyle::Success),                    
                        ])];
                
                        poise::CreateReply::default()
                            .content(format!("Instructions: website {}", website_id))
                            .components(components)
                            .ephemeral(true)
                    };
                    if let Some(row) = sqlx::query!("SELECT website_id FROM deposit WHERE discord_id=$1", discord_id).fetch_optional(conn).await? {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("You already have an ongoing deposit process on website {}", row.website_id));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    }
                    sqlx::query!("INSERT INTO deposit(website_id, discord_id) VALUES ($1,$2)", website_id, discord_id).execute(conn).await?;
        
                    let mut data = serenity::CreateInteractionResponseMessage::new();
                    data = reply.to_slash_initial_response(data);
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                } else if custom_id.starts_with("deposit-cancel") {
                    let Some(_) = sqlx::query!("SELECT website_id FROM deposit WHERE discord_id=$1", discord_id).fetch_optional(conn).await? else {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content("You have no ongoing deposit process");
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                        return Ok(())
                    };
                    sqlx::query!("DELETE FROM deposit WHERE discord_id=$1 AND website_id=$2", discord_id, website_id).execute(conn).await?;
                    let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("Your deposit process on website {} was cancelled", website_id));
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                } else if custom_id.starts_with("deposit-finish") {
                    dbg!("bababoey");
                    if sqlx::query!("UPDATE deposit SET is_check=TRUE WHERE discord_id=$1 AND website_id=$2 ", discord_id, website_id).execute(conn).await?.rows_affected() == 1 {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("Your deposit on website {} was just marked as finished", website_id));
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                    } else {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content("You have no ongoing deposit process");
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}