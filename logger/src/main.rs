use db::{DataEntry, NutStatus, init_db};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{process, thread};

const IP: &str = "192.168.0.24";
const PORT: &str = "3493";
const POLLING_RATE: u64 = 5;

fn get_ups_data() -> Result<DataEntry, String> {
    // if this funciton errors out it shoudl log it and keep the process running
    //get the data from ups
    let stream = TcpStream::connect(format!("{}:{}", IP, PORT));

    let mut stream = match stream {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to connect to UPS: {}", e));
        }
    };
    let ups_variables = stream.write_all(b"LIST VAR mgeups\n");

    match ups_variables {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Failed getting variables from UPS: {}", e));
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

    let mut charge: bool = false;
    let mut load: bool = false;
    let mut status: bool = false;
    let mut outlet: bool = false;

    for line in reader.lines() {
        let line = line.expect("Error reading from stream");
        if line.starts_with("END LIST") {
            break;
        }

        match line {
            l if l.contains("battery.charge ") => {
                charge = true;
                let value = l.split_whitespace().last().expect("Failed to parse charge");
                data.charge = match value.trim_matches('"').parse() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Error formating battery: {}", e));
                    }
                }
            }
            l if l.contains("ups.load ") => {
                load = true;
                let value = l.split_whitespace().last().expect("Failed to parse load");
                data.load = match value.trim_matches('"').parse() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Error formating charge: {}", e));
                    }
                }
            }
            l if l.contains("ups.status ") => {
                status = true;
                let value = l.split_whitespace().last().expect("Failed to parse status");
                data.status = match value.trim_matches('"') {
                    "OL" => NutStatus::OL,
                    "OB" => NutStatus::OB,
                    "LB" => NutStatus::LB,
                    other => return Err(format!("Error parsing status: {}", other)),
                }
            }
            l if l.contains("outlet.1.status ") => {
                outlet = true;
                let value = l.split_whitespace().last().expect("Failed to parse Outlet");
                data.outlet = match value.trim_matches('"') {
                    "on" => true,
                    "off" => false,
                    other => return Err(format!("Error parsing other: {}", other)),
                }
            }
            _ => {
                //discard line
            }
        };
    }

    if charge && load && status && outlet {
        return Ok(data);
    }

    return Err(String::from("Error reading one of the lines"));
}

fn main() {
    println!("Connecting to DB...");

    let conn = init_db();

    loop {
        let data = get_ups_data();

        match data {
            Ok(data) => {
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
                #[cfg(debug_assertions)]
                println!("{data:?}");
            }
            Err(e) => {
                println!("{e:?}");
            }
        }

        thread::sleep(Duration::from_secs(POLLING_RATE));
    }
}
