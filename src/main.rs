use rusqlite::Connection;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

const IP: &str = "192.168.0.24";
const PORT: &str = "3493";

#[derive(Debug, Clone, Copy)]
enum NutStatus {
    OL = 0,
    OB = 1,
    LB = 2,
}

#[derive(Debug)]
struct DataEntry {
    charge: u8,
    load: u16,
    status: NutStatus,
    outlet: bool,
    timestamp: u64,
}

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
        "CREATE TABLE IF NOT EXISTS data_log (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            charge    INTEGER NOT NULL,
            load      INTEGER NOT NULL,
            status    INTEGER NOT NULL,
            outlet    INTEGER NOT NULL,
            timestamp INTEGER NOT NULL
        );",
        (),
    );

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
            l if l.contains("battery.charge ") => {
                let value = l.split_whitespace().last().expect("Failed to parse charge");
                data.charge = value
                    .trim_matches('"')
                    .parse()
                    .expect("Failed to parse charge");
            }
            l if l.contains("ups.load ") => {
                let value = l.split_whitespace().last().expect("Failed to parse load");
                data.load = value
                    .trim_matches('"')
                    .parse()
                    .expect("Failed to parse charge");
            }
            l if l.contains("ups.status ") => {
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
            l if l.contains("outlet.1.status ") => {
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

    let _ = conn.execute(
        "INSERT INTO data_log (charge, load, status, outlet, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            data.charge,
            data.load,
            data.status as u8,
            data.outlet as u8,
            data.timestamp as i64,
        ),
    );

    println!("{data:?}");
}
