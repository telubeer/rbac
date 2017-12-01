extern crate serde_json;
extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate router;
extern crate time;
extern crate sys_info;

use self::serde_json::Value as JsonValue;
use self::iron::prelude::*;
use self::iron::headers::ContentType;
use self::iron::status;
use self::persistent::{State, Read};
use self::iron::typemap::Key;
use self::router::Router;
use self::time::now;

use mysql;
use mysql::Pool;
use mods::loader::{load, load_items};
use mods::rbac::{Data, UserId};

impl Key for Data { type Value = Data; }

#[derive(Copy, Clone)]
pub struct DbPool;
impl Key for DbPool {
    type Value = mysql::Pool;
}
#[derive(Copy, Clone)]
pub struct  Uptime;
impl Key for Uptime { type Value = i64; }

fn handle(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<State<Data>>().unwrap();
    let data = arc.read().unwrap();

    let mut out: JsonValue = json!([]);
    let body = req.get::<bodyparser::Raw>();
    match body {
        Ok(Some(body)) => {
            let items:JsonValue = serde_json::from_str(&body).unwrap();
            for item in items.as_array().unwrap().iter() {
                let user_id: UserId = item["user_id"].to_string().parse().unwrap();
                let action = item["action"].as_str().unwrap();
                let params = item["params"].as_array().unwrap();
                let mut res: JsonValue = json!([]);
                for param in params.iter() {
                    let result = data.check_access(
                        user_id,
                        action.to_string(),
                        &param
                    );
                    let _ = res.as_array_mut().unwrap().push(JsonValue::Bool(result));
                }
                let _ = out.as_array_mut().unwrap().push(res);
            }
        }
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }
    Ok(Response::with((ContentType::json().0, status::Ok, out.to_string())))
}

fn reload(req: &mut Request) -> IronResult<Response> {


    let pool = &req.get::<Read<DbPool>>().unwrap();
    let (map, mut items, mut parents, mut assignments) = load_items(pool);
    let new_data = load(pool.as_ref());

    let arc = req.get::<State<Data>>().unwrap();
    let mut data= arc.write().unwrap();
    *data = new_data;

    let data:JsonValue = json!({
        "status" : "ok",
        "users" : data.assignments.len(),
    });
    Ok(Response::with((ContentType::json().0, status::Ok, data.to_string())))
}

fn health(req: &mut Request) -> IronResult<Response> {
    let pool = &req.get::<Read<DbPool>>().unwrap();
    let hostname = mark_as_running(pool.as_ref());

    let start_time = &req.get::<Read<Uptime>>().unwrap();
    let uptime = now().to_timespec().sec - start_time.as_ref();

    let data:JsonValue = json!({
        "status" : "ok",
        "uptime" : uptime,
        "hostname" : hostname,
    });
    Ok(Response::with((ContentType::json().0, status::Ok, data.to_string())))
}

pub fn run(listen: &str, data: Data, pool: Pool) {
    let start_time = now().to_timespec().sec;
    let hostname = mark_as_running(&pool);

    let mut router = Router::new();
    router.post("/check", handle, "check");
    router.post("/reload", reload, "reload");
    router.get("/health", health, "health");

    let mut chain = Chain::new(router);
    chain.link(State::<Data>::both(data));
    chain.link(Read::<DbPool>::both(pool));
    chain.link(Read::<Uptime>::both(start_time));

    println!("start listening on {} hostname {}", &listen, hostname);
    Iron::new(chain).http(listen).unwrap();
}

fn mark_as_running(pool: &Pool) -> String {
    let hostname = self::sys_info::hostname().unwrap();
    let mut stmt = pool
        .prepare("INSERT INTO ngs_regionnews.auth_instances (ip)\
         VALUES (:hostname) \
         ON DUPLICATE KEY UPDATE time=NOW()").unwrap();
    stmt.execute(params!{"hostname" => &hostname}).unwrap();
    hostname
}