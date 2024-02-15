use actix_web::{get, web, App, HttpResponse, HttpServer, HttpRequest, Responder};
use serde::Serialize;
use serde::Deserialize;
use serde_json;
use std::{
    net::SocketAddr,
    sync::Mutex,
};


//Hashmap to store server data
#[derive(Serialize, Debug)]
struct ServerObject {
    ip: Option<SocketAddr>,
    port: i32,
}


//List struct that builds our server vector with a mutex
#[derive(Serialize)]
struct ServerList {
    list: Mutex<Vec<ServerObject>>,
}


//Sanitize query params
#[derive(Deserialize)]
struct ServerMeta {
    port: i32,
}


//Health check
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("I am going insane")
}


//Endpoint to handle server registration
#[get("/announce")]
async fn announce(data: web::Data<ServerList>, servermeta: web::Query<ServerMeta>, req: HttpRequest) -> impl Responder {
    
    let mut serverlist = data.list.lock().unwrap();

    let serverinstance = ServerObject {
        ip: req.peer_addr(),
        port: servermeta.port,
    };

    serverlist.push(serverinstance);
    HttpResponse::Ok().body("Something happened")
}


//List all servers in serverlist
#[get("/list")]
async fn list(data: web::Data<ServerList>) -> impl Responder {
    let serverlist = data.list.lock().unwrap();

    for server in serverlist.iter() {
        println!("{:#?}", server);
    }

    HttpResponse::Ok().body("json")
}


//Main entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let serverlist = web::Data::new(ServerList {
        list: Mutex::new(Vec::<ServerObject>::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(serverlist.clone())
            .service(hello)
            .service(announce)
            .service(list)
    })

    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
