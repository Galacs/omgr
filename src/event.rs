use poise::serenity_prelude::{self as serenity, FullEvent};

use crate::{Data, Error, embeds::{get_deposit_edit_message, update_latest_embed}};

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
                    if sqlx::query!("INSERT INTO deposit(website_id, discord_id, interaction_token) VALUES ($1,$2,$3)", website_id, discord_id, &interaction.token).execute(conn).await.is_err() {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("A deposit process is already going on website {}", website_id));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    };
                    let mut data = serenity::CreateInteractionResponseMessage::new();
                    data = reply.to_slash_initial_response(data);
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} initiated a deposit on website {}", interaction.user, website_id)).await?;
                    // Update out-of-date deposit embed
                    // let reply = get_deposit_edit_message(conn).await?;
                    // interaction.message.clone().edit(&ctx.http, reply).await?;
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
                    // dbg!(interaction.message)
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} cancelled a deposit on website {}", interaction.user, website_id)).await?;
                    // Update out-of-date withdraw embed
                    // update_latest_embed(conn, &ctx.http).await?;
                } else if custom_id.starts_with("deposit-finish") {
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
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} finished a deposit on website {}", interaction.user, website_id)).await?;
                }
            } else if custom_id.starts_with("withdraw") {
                let Some(website_id) = &interaction.data.custom_id.split('-').nth(2) else {
                    return Ok(());
                };
                if custom_id.starts_with("withdraw-init") {
                    if let Some(row) = sqlx::query!("SELECT website_id FROM withdraw WHERE discord_id=$1", discord_id).fetch_optional(conn).await? {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("You already have an ongoing withdraw process on website {}", row.website_id));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    }
                    // Modal
                    let modal = serenity::CreateQuickModal::new("Withdraw Process")
                        .timeout(std::time::Duration::from_secs(600))
                        .short_field("Amount");
                    let response = interaction.quick_modal(ctx, modal).await?.unwrap();
                    let amount: i32 = response.inputs[0].parse()?;


                    let stock = sqlx::query!("SELECT stock FROM stock WHERE website_id=$1", website_id).fetch_one(conn).await?.stock;   
                    if stock < amount {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("This website doesn't have enough stock left, it only has {} left", stock));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        response.interaction.create_response(&ctx.http, builder).await?;
                        return Ok(())
                    }
                    let balance = sqlx::query!("SELECT balance FROM balances WHERE discord_id=$1", discord_id).fetch_one(conn).await?.balance;   
                    let tax = sqlx::query!("SELECT rate from tax WHERE tax='withdraw' AND website_id=$1", website_id).fetch_one(conn).await?.rate;

                    let r#final = (amount as f32 * (1.0+tax/100.0)).floor() as i32;
                    if balance < r#final.into() {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("You do not have enough balance, you need {}", r#final));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        response.interaction.create_response(&ctx.http, builder).await?;
                        return Ok(())
                    }

                    if sqlx::query!("INSERT INTO withdraw(website_id, discord_id, interaction_token, amount) VALUES ($1,$2,$3,$4)", website_id, discord_id, &interaction.token, r#final).execute(conn).await.is_err() {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("A withdraw process is already going on website {}", website_id));
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        response.interaction.create_response(&ctx.http, builder).await?;
                        return Ok(())
                    };
                    let reply = {
                        let components = vec![serenity::CreateActionRow::Buttons(vec![
                            serenity::CreateButton::new(format!("withdraw-cancel-{}", website_id))
                                .label("Cancel withdraw")
                                .style(poise::serenity_prelude::ButtonStyle::Secondary),
                            serenity::CreateButton::new(format!("withdraw-finish-{}", website_id))
                                .label("Finish withdraw")
                                .style(poise::serenity_prelude::ButtonStyle::Success),                    
                        ])];
                
                        poise::CreateReply::default()
                            .content(format!("Instructions: website {}. Withdraw costs {} and {} will be given at a tax of {}%", website_id, r#final, amount, tax))
                            // .components(components)
                            .ephemeral(true)
                    };
                    let mut data = serenity::CreateInteractionResponseMessage::new();
                    data = reply.to_slash_initial_response(data);
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    response.interaction.create_response(&ctx.http, builder).await?;
                    sqlx::query!("UPDATE balances SET balance = balance - $2 WHERE discord_id = $1", discord_id, r#final as i64).execute(conn).await?;
                    sqlx::query!("UPDATE stock SET stock = stock - $2 WHERE website_id = $1", website_id, amount as i64).execute(conn).await?;
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} initiated a withdraw on website {} for an amount of {}", interaction.user, website_id, amount)).await?;
                    // Update out-of-date withdraw embed
                    // let reply = get_deposit_edit_message(conn).await?;
                    // interaction.message.clone().edit(&ctx.http, reply).await?;

                    // Quick route to finish pasted
                    if sqlx::query!("UPDATE withdraw SET is_check=TRUE WHERE discord_id=$1 AND website_id=$2 ", discord_id, website_id).execute(conn).await?.rows_affected() == 1 {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("Your withdraw on website {} was just marked as finished", website_id));
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                    } else {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content("You have no ongoing withdraw process");
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    }
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} finished a withdraw on website {}", interaction.user, website_id)).await?;
                } else if custom_id.starts_with("withdraw-cancel") {
                    let Some(_) = sqlx::query!("SELECT website_id FROM withdraw WHERE discord_id=$1", discord_id).fetch_optional(conn).await? else {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content("You have no ongoing withdraw process");
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                        return Ok(())
                    };
                    sqlx::query!("DELETE FROM withdraw WHERE discord_id=$1 AND website_id=$2", discord_id, website_id).execute(conn).await?;
                    let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("Your withdraw process on website {} was cancelled", website_id));
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                    // dbg!(interaction.message);
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} cancelled a withdraw on website {}", interaction.user, website_id)).await?;
                    // Update out-of-date withdraw embed
                    // update_latest_embed(conn, &ctx.http).await?;
                } else if custom_id.starts_with("withdraw-finish") {
                    if sqlx::query!("UPDATE withdraw SET is_check=TRUE WHERE discord_id=$1 AND website_id=$2 ", discord_id, website_id).execute(conn).await?.rows_affected() == 1 {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("Your withdraw on website {} was just marked as finished", website_id));
                    let builder = serenity::CreateInteractionResponse::Message(data);
                    interaction.create_response(&ctx.http, builder).await?;
                    } else {
                        let data = serenity::CreateInteractionResponseMessage::new().ephemeral(true)
                            .content("You have no ongoing withdraw process");
                        let builder = serenity::CreateInteractionResponse::Message(data);
                        interaction.create_response(&ctx.http, builder).await?;
                    }
                    // Log to discord channel
                    dslog::send_log_to_discord(&ctx.http, conn, interaction.guild_id.ok_or("in pm")?, &format!("{} finished a withdraw on website {}", interaction.user, website_id)).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}