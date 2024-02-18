use actix_web::{get, web, App, HttpResponse, HttpServer, HttpRequest, Responder};
use std::net::IpAddr;
use serde::Serialize;
use serde::Deserialize;
use serde_json;
use std::{
    sync::Mutex,
    time::SystemTime
};


//Struct to store server data
#[derive(Serialize, Debug)]
struct ServerObject {
    pub ip: IpAddr,
    pub port: i32,
    pub time: SystemTime,
}


//Server json 
#[derive(Serialize, Debug)]
struct ServerJson {
    ipport: String
}


//List struct that builds our server vector with a mutex
#[derive(Serialize)]
struct ServerList {
    pub list: Mutex<Vec<ServerObject>>,
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
    let val = req.peer_addr();

    let serverinstance = ServerObject {
        ip: val.expect("Someone connected without an ip!!1").ip(),
        port: servermeta.port,
        time: SystemTime::now(),

    };

    //Check for existing server ip/port in the vector 
    if serverlist.len() > 0 {
        for server in serverlist.iter_mut() {
            if (server.ip == val.expect("").ip()) && (server.port == servermeta.port) {
                server.ip = val.expect("").ip();
                server.port = servermeta.port;
                server.time = SystemTime::now();
            } else {
                serverlist.push(serverinstance);
            }
        }
    } else {
        serverlist.push(serverinstance);
    }

    HttpResponse::Ok().json("success")
}


//List all servers in serverlist
#[get("/list")]
async fn list(data: web::Data<ServerList>) -> impl Responder {
    let mut serverlist = data.list.lock().unwrap();
    
    //Prune all servers in the list outside the ttk threshold
    //Much faster than .remove() as rust doesnt have to iterate over the vector for each element then shift indexes after removal
    serverlist.retain(|server| SystemTime::now().duration_since(server.time).unwrap().as_secs() <= 60);
    println!("{:#?}", serverlist);

    //ToDo: Copy all of our server records to new structs that are more friendly for json formatting then ship
    if serverlist.len() > 0 {
        for server in serverlist.iter_mut() {
            let server_ip = server.ip.to_string();
            let server_port = server.port.to_string();
            let server_address = format!("{}:{}", server_ip, server_port);
            
            let server_instance = ServerJson {
                ipport: server_address
            };

            println!("{:#?}", server_instance);
        }

    }

    let response_body = serde_json::to_string(&*serverlist).unwrap();

    HttpResponse::Ok().body(response_body)
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
