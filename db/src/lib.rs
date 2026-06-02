use directories::ProjectDirs;
use rusqlite::Connection;
use std::{fs::create_dir_all, process};

const DB_NAME: &str = "rupsuidata.db";

#[derive(Debug, Clone, Copy)]
pub enum NutStatus {
    OL = 0,
    OB = 1,
    LB = 2,
}

#[derive(Debug)]
pub struct DataEntry {
    pub charge: u8,
    pub load: u16,
    pub status: NutStatus,
    pub outlet: bool,
    pub timestamp: u64,
}

pub fn init_db() -> Connection {
    let project = ProjectDirs::from("com", "sandysaint", "rupsui")
        .expect("could not determine home directory");

    let project_dir = project.data_dir();

    create_dir_all(project_dir).expect(&format!(
        "Failed to create data directory on {}",
        project_dir.display()
    ));

    let conn = Connection::open(project_dir.join(DB_NAME));

    println!("DB location: {}", project_dir.display());

    let conn = match conn {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect to DB: {}", e);
            process::exit(1)
        }
    };

    match conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA busy_timeout=5000;
        ",
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to set WAL mode: {}", e);
            process::exit(1);
        }
    };

    let create = conn.execute(
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

    match create {
        Ok(_) => {
            println!("Created table data_log");
        }
        Err(e) => {
            println!("Error creating table: {}", e);
        }
    }

    return conn;
}
