use rusqlite::Connection;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

const IP: &str = "localhost";
const PORT: &str = "3493";

#[derive(Debug)]
enum NutStatus {
    OL,
    OB,
    LB,
}

#[derive(Debug)]
struct DataEntry {
    charge: u8,
    load: u16,
    status: NutStatus,
    outlet: bool,
    timestamp: u64,
}

fn init_db() {}

fn get_ups_data() {}

fn main() {
    println!("Connecting to DB...");
    let conn = Connection::open("my_db.db");

    let conn = match conn {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect {}", e);
            return;
        }
    };

    let _ = conn.execute(
        "CREATE TABLE person (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
    )",
        (),
    );

    let insert = conn.execute("INSERT INTO person (name) VALUES (?1)", ("mike",));

    let _ = match insert {
        Ok(i) => i,
        Err(e) => {
            println!("Failed to insert: {}", e);
            return;
        }
    };

    //get the data from ups
    let stream = TcpStream::connect(format!("{}:{}", IP, PORT));

    let mut stream = match stream {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect to UPS: {}", e);
            return;
        }
    };
    let ups_variables = stream.write_all(b"LIST VAR mgeups\n");

    match ups_variables {
        Ok(v) => v,
        Err(e) => {
            println!("Failed getting variables from UPS: {}", e);
            return;
        }
    };

    let reader = BufReader::new(&stream);

    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut data = DataEntry {
        timestamp: now,
        charge: 0,
        load: 0,
        status: NutStatus::OL,
        outlet: true,
    };

    for line in reader.lines() {
        let line = line.expect("Error reading from stream");
        if line.starts_with("END LIST") {
            break;
        }

        match line {
            l if l.contains("battery.charge") => {
                let value = l.split_whitespace().last().expect("Failed to parse charge");
                data.charge = value
                    .trim_matches('"')
                    .parse()
                    .expect("Failed to parse charge");
            }
            l if l.contains("ups.load") => {
                let value = l.split_whitespace().last().expect("Failed to parse load");
                data.load = value
                    .trim_matches('"')
                    .parse()
                    .expect("Failed to parse charge");
            }
            l if l.contains("ups.status") => {
                let value = l.split_whitespace().last().expect("Failed to parse status");
                data.status = match value.trim_matches('"') {
                    "OL" => NutStatus::OL,
                    "OB" => NutStatus::OB,
                    "LB" => NutStatus::LB,
                    _ => {
                        println!("Error Parsing Status");
                        return;
                    }
                }
            }
            l if l.contains("outlet.1.status") => {
                let value = l.split_whitespace().last().expect("Failed to parse Outlet");
                data.outlet = match value.trim_matches('"') {
                    "on" => true,
                    "off" => false,
                    _ => {
                        println!("Error parsing Outlet");
                        return;
                    }
                }
            }
            _ => {}
        };
    }

    println!("structed data {data:?}");
}
