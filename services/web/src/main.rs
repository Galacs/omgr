use actix_web::{web, App, HttpServer, Responder, get, HttpResponse, post};
use sqlx::{Pool, Postgres, PgPool};
use serde::{Serialize, Deserialize};

use poise::serenity_prelude::{self as serenity, EditMessage};

pub struct Data(Pool<Postgres>, std::sync::Arc<serenity::Http>);

#[derive(Serialize, Deserialize, Debug)]
struct Deposit {
    discord_id: i64,
    website_id: String,
}

#[get("/deposits")]
async fn get_deposits(data: web::Data<Data>) -> impl Responder {
    let conn = &data.as_ref().0;
    // Get deposit processe with a check status
    let Ok(rows) = sqlx::query!("SELECT discord_id,website_id FROM deposit WHERE is_check=TRUE").fetch_all(conn).await else {
        return HttpResponse::InternalServerError().body("DB error")
    };

    let deposits: Vec<_> = rows.iter().map(|r| Deposit { discord_id: r.discord_id, website_id: r.website_id.clone() }).collect();

    HttpResponse::Ok().json(deposits)
}

// Temporary
pub async fn create_deposit_embed_message(conn: &Pool<Postgres>) -> Result<poise::CreateReply, anyhow::Error> {
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
            .content("Deposit process explanation")
            .components(components))
}

pub async fn get_deposit_edit_message(conn: &Pool<Postgres>) -> Result<EditMessage, anyhow::Error> {
    let builder = create_deposit_embed_message(conn).await?;
    Ok(EditMessage::default().content("Deposit process explanation").components(builder.components.unwrap()))
}

#[post("/deposits")]
async fn post_deposits(data: web::Data<Data>, deposits: web::Json<Vec<Deposit>>) -> impl Responder {
    let conn = &data.as_ref().0;
    let http = &data.as_ref().1;
    for deposit in &deposits.0 {
        let Ok(query) = sqlx::query!("DELETE FROM deposit WHERE discord_id=$1 AND website_id=$2 AND is_check=TRUE", deposit.discord_id, deposit.website_id).execute(conn).await else {
            return HttpResponse::InternalServerError().body("DB error")
        };
        if query.rows_affected() < 1 {
            return HttpResponse::InternalServerError().body(format!("The deposit process: {}, website {} doesn't exist or isn't in check state", deposit.discord_id, deposit.website_id))
        }
    }
    if let Ok(Some(embed_id)) = sqlx::query!("SELECT message_id,channel_id FROM embed").fetch_optional(conn).await {
        let message_id = serenity::MessageId::new(embed_id.message_id as u64);
        let channel_id = serenity::ChannelId::new(embed_id.channel_id as u64);
        if let Ok(mut msg) = http.get_message(channel_id, message_id).await {
            if let Ok(builder) = get_deposit_edit_message(conn).await {
                let _ = msg.edit(http, builder).await;
            }
        }
    };
    HttpResponse::Ok().body(format!("{} deposits were marked as complete", deposits.0.len()))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // DB
    let database_url = std::env::var("DATABASE_URL").expect("Expected a database url in the environment");
    let conn = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&conn).await?;

    // Discord api clint
    let app_id = serenity::ApplicationId::new(std::env::var("DICSORD_APP_ID").expect("missing DICSORD_APP_ID").parse()?);
    let client = serenity::Client::builder(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), serenity::GatewayIntents::non_privileged()).application_id(app_id).await.expect("Err creating client");

    HttpServer::new(move || {
        App::new().service(get_deposits).service(post_deposits).app_data(web::Data::new(Data(conn.clone(), client.http.clone())))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await?;
    Ok(())
}