use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::ConnectionInfo;
use actix_web::HttpRequest;
use actix_cors::Cors;
use awc::Client;
use std::time::Duration;
use serde::Serialize;
use serde::Deserialize;

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


//Check existance of game server headers
fn get_server_header<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    req.headers().get("User-Agent")?.to_str().ok()
}


//Endpoint to handle server registration
#[get("/announce")]
async fn announce(data: web::Data<ServerList>, servermeta: web::Query<ServerMeta>, conn: ConnectionInfo, req: HttpRequest) -> impl Responder {
    let mut serverlist = data.list.lock().unwrap();
    let real_ip = conn.realip_remote_addr().expect("ope").to_string();

    let serverinstance = ServerObject {
        ip: real_ip.clone(),
        port: servermeta.port,
        time: SystemTime::now(),

    };


    //Verify the server endpoint is accessible (This cuts out servers with bad port forwards or malicious annoucements)
    let client = Client::new();
    let server_endpoint = format!("http://{}:{}", real_ip.clone(), servermeta.port);
    let endpoint_result = client.get(server_endpoint).timeout(Duration::from_secs(3)).send().await;

    if endpoint_result.is_err() {
        println!("Bad Server");
        return HttpResponse::Ok().json("Bad Server");
    }


    //Check game server headers
    let header_vec = vec!["ElDewrito/0.6.1.0", "ElDewrito/0.5.1.1"];
    if let Some(server_header) = get_server_header(&req) {
        if !header_vec.contains(&server_header) {
            println!("{}", server_header);
            return HttpResponse::Ok().json("Bad Headers");
        }
    } else {
        return HttpResponse::Ok().json("Bad Headers");
    }


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
    //Retain creates a copy of only the records we want based on the operation provided and destroys the old vector
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
        //We ball
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(serverlist.clone())
            .service(hello)
            .service(announce)
            .service(list)
    })

    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
