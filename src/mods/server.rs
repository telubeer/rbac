extern crate json;
extern crate iron;
extern crate bodyparser;
extern crate persistent;

use json::JsonValue;
use self::iron::prelude::*;
use self::iron::headers::ContentType;
use self::iron::status;
use self::persistent::Read;
use self::iron::typemap::Key;
use mods::rbac::Data;

impl Key for Data { type Value = Data; }

fn handle(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<Data>>().unwrap();
    let data = arc.as_ref();

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

pub fn run(listen: &str, data: Data) {
    let mut chain = Chain::new(handle);
    chain.link(Read::<Data>::both(data));
    Iron::new(chain).http(listen).unwrap();
}