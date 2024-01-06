use actix_web::{web, App, HttpServer, Responder, get, HttpResponse, post};
use sqlx::{Pool, Postgres, PgPool};
use serde::{Serialize, Deserialize};

pub struct Data(Pool<Postgres>);

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

#[post("/deposits")]
async fn post_deposits(data: web::Data<Data>, deposits: web::Json<Vec<Deposit>>) -> impl Responder {
    let conn = &data.as_ref().0;
    for deposit in &deposits.0 {
        let Ok(query) = sqlx::query!("DELETE FROM deposit WHERE discord_id=$1 AND website_id=$2 AND is_check=TRUE", deposit.discord_id, deposit.website_id).execute(conn).await else {
            return HttpResponse::InternalServerError().body("DB error")
        };
        if query.rows_affected() < 1 {
            return HttpResponse::InternalServerError().body(format!("The deposit process: {}, website {} doesn't exist or isn't in check state", deposit.discord_id, deposit.website_id))
        }
    }
    HttpResponse::Ok().body(format!("{} deposits were marked as complete", deposits.0.len()))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // DB
    let database_url = std::env::var("DATABASE_URL").expect("Expected a database url in the environment");
    let conn = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&conn).await?;


    HttpServer::new(move || {
        App::new().service(get_deposits).service(post_deposits).app_data(web::Data::new(Data(conn.clone())))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await?;
    Ok(())
}