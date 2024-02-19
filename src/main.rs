use actix_web::{get, web, App, HttpResponse, HttpServer, HttpRequest, Responder};
use actix_web::dev::ConnectionInfo;
use std::net::IpAddr;
use serde::Serialize;
use serde::Deserialize;
use serde_json;
use std::{
    sync::Mutex,
    time::SystemTime
};


#[derive(Serialize, Debug, Clone)]
struct ServerObject {
    pub ip: String,
    pub port: u16,
    pub time: SystemTime,
}


#[derive(Serialize)]
struct Result {
    pub listVersion: u8,
    pub code: u8,
    pub servers: Vec<String>,
    pub msg: String

}

#[derive(Serialize)]
pub struct Response {
    result: Result
}


#[derive(Serialize, Debug)]
struct ServerJson {
    ipport: String
}


#[derive(Serialize)]
struct ServerList {
    pub list: Mutex<Vec<ServerObject>>,
}


#[derive(Deserialize)]
struct ServerMeta {
    port: u16
}


//Health check
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("I am going insane")
}


//Endpoint to handle server registration
#[get("/announce")]
async fn announce(data: web::Data<ServerList>, servermeta: web::Query<ServerMeta>, req: HttpRequest, conn: ConnectionInfo) -> impl Responder {
    let mut serverlist = data.list.lock().unwrap();
    //let val = req.peer_addr();
    let real_ip = conn.realip_remote_addr().expect("ope").to_string();

    let serverinstance = ServerObject {
        ip: real_ip.clone(),
        port: servermeta.port,
        time: SystemTime::now(),

    };

    //Check for existing server ip/port in the vector
    if serverlist.len() > 0 {
        if serverlist.iter().any(|server| (server.ip == real_ip) && (server.port == servermeta.port)) {
            for i in 1..serverlist.len() {
                if (serverlist[i].ip == real_ip) && (serverlist[i].port == servermeta.port) {
                    serverlist[i].ip = real_ip.clone();
                    serverlist[i].port = servermeta.port;
                    serverlist[i].time = SystemTime::now();
                };
            }
        } else {
            serverlist.push(serverinstance.clone());
        }
    } else {
        serverlist.push(serverinstance.clone());
    }

    HttpResponse::Ok().json("success")
}


//List all servers in serverlist
#[get("/list")]
async fn list(data: web::Data<ServerList>) -> impl Responder {
    let mut serverlist = data.list.lock().unwrap();
    let mut formattedlist = Vec::new();

    //Prune all servers in the list outside the ttk threshold
    //Much faster than .remove() as rust doesnt have to iterate over the vector for each element then shift indexes after removal
    serverlist.retain(|server| SystemTime::now().duration_since(server.time).unwrap().as_secs() <= 300);
    println!("{:#?}", serverlist);

    //ToDo: Copy all of our server records to new structs that are more friendly for json formatting then ship
    if serverlist.len() > 0 {
        for server in serverlist.iter_mut() {
            let server_ip = server.ip.to_string();
            let server_port = server.port.to_string();
            let server_address = format!("{}:{}", server_ip, server_port);

            formattedlist.push(server_address);
        }
    }

    let result = Result {
        listVersion: 1,
        code: 0,
        servers: formattedlist,
        msg: "OK".to_string()
    };

    let response = Response{
        result
    };

    HttpResponse::Ok().json(response)
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

    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
