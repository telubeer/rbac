extern crate actix;
extern crate actix_web;
extern crate json;
extern crate time;
extern crate sys_info;
extern crate futures;
extern crate tokio_core;

use json::JsonValue;
use std::sync::{Arc, RwLock};
use self::time::now;
use self::futures::sync::mpsc::Sender;
use self::futures::{Future, Sink};
use self::futures::future::{FutureResult, result};
use self::tokio_core::reactor::{Remote};

use mods::rbac::{Data, UserId};

use std::str;
use self::actix::*;
use self::actix_web::*;

#[derive(Clone)]
struct AppState {
    data: Arc<RwLock<Data>>,
    uptime: i64,
    tx: Sender<i8>,
    remote: Remote,
}

fn handle(req: HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>>  {
    let data = req.state().data.clone();
    req.body()
        .from_err()
        .map(|raw_body| {
            str::from_utf8(&raw_body).unwrap().to_string()
        })
        .and_then(move | body| {
            let items = json::parse(&body).unwrap();
            let mut out: JsonValue = array![];
            for item in items.members() {
                let user_id: UserId = item["user_id"].to_string().parse().unwrap();
                let action = &item["action"];
                let params = &item["params"];
                let mut res: JsonValue = array![];
                for param in params.members() {
                    let result = data.read().unwrap()
                        .check_access(
                            user_id,
                            action.to_string(),
                            &param,
                        );
                    let _ = res.push(result);
                }
                let _ = out.push(res);
            }
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("application/json")
               // .header("X-Data-Timestamp", data.read().unwrap().timestamp.to_owned())
                .body(out.dump()).unwrap())
        })
        .responder()
}

fn reload(req: HttpRequest<AppState>) -> Result<HttpResponse> {
    let tx = req.state().tx.clone();
    let remote = req.state().remote.clone();
    remote.spawn(move|_| {
        tx.send(2)
            .then(|tx| {
                match tx {
                    Ok(_tx) => {
                        debug!("send work");
                        Ok(())
                    }
                    Err(e) => {
                        error!("send work failed! {:?}", e);
                        Err(())
                    }
                }
            })
    });

    let data = object! {
        "status" => "ok",
    };
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(json::stringify(data)).unwrap())
}

fn health(req: HttpRequest<AppState>) -> Result<HttpResponse> {
    let data = req.state().data.as_ref().read().unwrap();
    let start_time = req.state().uptime;
    let uptime = now().to_timespec().sec - start_time;
    let hostname = self::sys_info::hostname().unwrap();
    let data = object! {
        "status" => "ok",
        "uptime" => uptime,
        "hostname" => hostname,
        "data_timestamp" => data.timestamp
    };
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(json::stringify(data)).unwrap())
}

pub fn run(listen: &str, data: Arc<RwLock<Data>>, tx: Sender<i8>, remote: Remote) {
    let uptime = now().to_timespec().sec;
    let sys = actix::System::new("rbac");

    let addr = HttpServer::new(
        move || Application::with_state(AppState{
            data: data.clone(),
            uptime,
            tx: tx.clone(),
            remote: remote.clone()
        })
            // enable logger
//            .middleware(middleware::Logger::default())
            .resource("/check", |r| r.method(Method::POST).f(handle))
            .resource("/reload", |r| r.method(Method::POST).f(reload))
            .resource("/health", |r| r.method(Method::GET).f(health))
        )
        .keep_alive(None)
        .shutdown_timeout(1)
        .bind(listen).unwrap()
        .start();

    println!("Started http server: {}", listen);
    let _ = sys.run();
}
