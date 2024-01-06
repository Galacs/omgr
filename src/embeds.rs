use poise::serenity_prelude::{self as serenity, GuildId, EditMessage};
use sqlx::{Postgres, Pool};

use crate::{Context, Error};

/// Creates the deposit embed
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
pub async fn create_deposit_embed(
    ctx: Context<'_>,
) -> Result<(), Error> { 
    let conn = &ctx.data().0;
    let reply = create_deposit_embed_message(conn).await?;
    let embed = ctx.send(reply).await?;
    let message_id: i64 = embed.message().await?.id.into();
    let channel_id: i64 = ctx.channel_id().into();
    let server_id: i64 = ctx.guild_id().unwrap_or(GuildId::new(10)).into();
    sqlx::query!("INSERT INTO embed(message_id,channel_id,server_id) VALUES ($1,$2,$3) ON CONFLICT(server_id) DO UPDATE SET message_id=$1", message_id, channel_id, server_id).execute(conn).await?;
    Ok(())
}

pub async fn create_deposit_embed_message(conn: &Pool<Postgres>) -> Result<poise::CreateReply, Error> {
    let rows = sqlx::query!("SELECT DISTINCT website_id FROM deposit").fetch_all(conn).await?;
    let get_disabled = |website_id: &str| {
        rows.iter().any(|r| r.website_id == website_id)
    };
    let get_style = |website_id: &str| {
        if get_disabled(website_id) {
            poise::serenity_prelude::ButtonStyle::Secondary
        } else {
            poise::serenity_prelude::ButtonStyle::Primary
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
    ])];

    Ok(poise::CreateReply::default()
            .content("Click the button below to open the modal")
            .components(components))
}

pub async fn get_deposit_edit_message(conn: &Pool<Postgres>) -> Result<EditMessage, Error> {
    let builder = create_deposit_embed_message(conn).await?;
    Ok(EditMessage::default().content("Click the button below to open the modal").components(builder.components.unwrap()))
}

/// Force updates a created deposit embed
#[poise::command(context_menu_command = "Update deposit embed", slash_command)]
pub async fn update_deposit_embed(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] mut msg: serenity::Message,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let int_reply = poise::CreateReply::default().content("The embed was updated").ephemeral(true);
    poise::send_reply(ctx, int_reply).await?;
    let reply = get_deposit_edit_message(conn).await?;
    msg.edit(ctx.http(), reply).await?;
    Ok(())
}