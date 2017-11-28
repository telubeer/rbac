extern crate json;
extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate router;

use json::JsonValue;
use self::iron::prelude::*;
use self::iron::headers::ContentType;
use self::iron::status;
use self::persistent::{State, Read};
use self::iron::typemap::Key;
use self::router::Router;

use mysql;
use mysql::Pool;
use mods::loader::load;
use mods::rbac::Data;

impl Key for Data { type Value = Data; }

#[derive(Copy, Clone)]
pub struct DbPool;
impl Key for DbPool {
    type Value = mysql::Pool;
}

fn handle(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<State<Data>>().unwrap();
    let data = arc.read().unwrap();

    let mut out: JsonValue = array![];
    let body = req.get::<bodyparser::Raw>();
    match body {
        Ok(Some(body)) => {
            let items = json::parse(&body).unwrap();
            for item in items.members() {
                let user_id = &item["user_id"];
                let action = &item["action"];
                let params = &item["params"];
                let mut res: JsonValue = array![];
                for param in params.members() {
                    let result = data.check_access(
                        user_id.to_string(),
                        action.to_string(),
                        &param
                    );
                    let _ = res.push(result);
                }
                let _ = out.push(res);
            }
        }
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }
    Ok(Response::with((ContentType::json().0, status::Ok, json::stringify(out))))
}

fn reload(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<State<Data>>().unwrap();
    let mut data= arc.write().unwrap();

    let pool = &req.get::<Read<DbPool>>().unwrap();
    *data = load(pool.as_ref());
    println!("loaded {:?}", data.assignments.len());
    Ok(Response::with((ContentType::json().0, status::Ok, json::stringify("reloaded successful"))))
}

pub fn run(listen: &str, data: Data, pool: Pool) {
    let mut router = Router::new();
    router.post("/check", handle, "check");
    router.post("/reload", reload, "reload");

    let mut chain = Chain::new(router);
    chain.link(State::<Data>::both(data));
    chain.link(Read::<DbPool>::both(pool));
    println!("start listening on {}", &listen);
    Iron::new(chain).http(listen).unwrap();
}