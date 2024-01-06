use poise::serenity_prelude as serenity;

use crate::{Context, Error};

/// Creates the deposit embed
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
pub async fn create_deposit_embed(
    ctx: Context<'_>,
) -> Result<(), Error> { 
    let reply = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("deposit-init-1")
                .label("Website 1")
                .style(poise::serenity_prelude::ButtonStyle::Success),
            serenity::CreateButton::new("deposit-init-2")
                .label("Website 2")
                .disabled(true)
                .style(poise::serenity_prelude::ButtonStyle::Success),
            serenity::CreateButton::new("deposit-init-3")
                .label("Website 3")
                .style(poise::serenity_prelude::ButtonStyle::Success)
        ])];

        poise::CreateReply::default()
            .content("Click the button below to open the modal")
            .components(components)
    };

    ctx.send(reply).await?;
    Ok(())
}
