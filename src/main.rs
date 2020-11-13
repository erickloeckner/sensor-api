#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

use std::convert::TryInto;
use std::time::SystemTime;
//use rusqlite::{params, Connection, Result};
use rocket_contrib::databases::rusqlite;
use rocket::request::Form;

#[database("sqlite_db")]
struct DbConn(rusqlite::Connection);

#[derive(Debug)]
struct Reading {
    //id: i32,
    time: u32,
    temp: u32,
    humid: u32,
}

#[derive(FromForm)]
struct FormReading {
    id: u32,
    temp: u32,
    humid: u32,
}

#[get("/")]
fn index() -> &'static str {
    "API index"
}

#[get("/get-sensor/<name>/<count>")]
fn get_sensor(conn: DbConn, name: String, count: u32) -> String {
    let mut query = conn.prepare("SELECT time, temp, humid FROM data LEFT JOIN sensors ON data.sensor = sensors.id WHERE sensors.name = ? ORDER BY time DESC LIMIT ?");
    match query {
        Ok(mut q) => {
            let results = q.query_map(&[&name, &count], |row| Reading {
                time: row.get(0),
                temp: row.get(1),
                humid: row.get(2),
            });
            match results {
                Ok(mut res) => {
                    let mut out = String::new();
                    for row in res {
                        match row {
                            Ok(v) => {
                                out.push_str(&format!("{},{},{}\n", v.time, v.temp, v.humid));
                            }
                            Err(e) => (),
                        }
                    }
                    // format!("{:?}", res.nth(0).unwrap())
                    out
                }
                Err(err) => {
                    format!("Error: {}", err)
                }
            }
        }
        Err(err) => {
            format!("Error: {}", err)
        }
    }
}

#[post("/set-sensor", data = "<reading>")]
fn set_sensor(conn: DbConn, reading: Form<FormReading>) -> &'static str {
    let ts: i64;

    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(val) => {
            ts = val.as_secs().try_into().unwrap();
        }
        Err(_err) => {
            ts = 0;
        }
    }

    match conn.execute(
        "INSERT INTO data (time, sensor, temp, humid) VALUES (?1, ?2, ?3, ?4)",
        &[&ts, &reading.id, &reading.temp, &reading.humid],
    ) {
        Ok(rows) => {
            "1"
        }
        Err(_err) => {
            "0"
        }
    }
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/api", routes![index, get_sensor, set_sensor])
        .launch();
}
