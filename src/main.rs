use csv::ReaderBuilder;
use itertools::Itertools;
use serde::Deserialize;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Debug, Deserialize)]
struct CSVRecord {
    user_id: String,
    user_nm: String,
    amt: Option<i64>,
}

#[derive(Debug)]
struct User {
    user_id: String,
    user_nm: String,
    amt: i64,
}
use rusqlite::Connection;

fn open_db() -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open("sqlite.db")?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "OFF")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;

    conn.execute(
        "create table if not exists user (
            user_id text primary key,
            user_nm text not null,
            amt integer not null
        )",
        [],
    )?;

    Ok(conn)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = open_db()?;
    
    let reject_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rejected_users.txt")?;

    ReaderBuilder::new()
        .has_headers(true)
        .from_path("users.csv")?
        .deserialize::<CSVRecord>()
        .filter_map(|res| match res {
            Ok(r) => match r.amt {
                Some(amt) => Some(User {user_id: r.user_id,
                                        user_nm: r.user_nm,
                                        amt}),
                None => {
                    let _ = writeln!(&reject_file, "REJECTED: Missing or invalid 'amt' value");
                    None
                }
            },
            Err(e) => {
                let _ = writeln!(&reject_file, "REJECTED: {}", e);
                None
            }
        })
        .chunks(10_000)
        .into_iter()
        .try_for_each(|chunk| bulk_insert(&mut conn, chunk, &reject_file))
}

fn main() {
    let _rc = run();
}

// SQLite's way to bulk insert records is to use a single commit at the end of
// transaction. INSER OR IGNORE statement means a record maybe rejected,
// but the batch continues and good rows are committed
fn bulk_insert(
    conn: &mut rusqlite::Connection,
    chunk: impl Iterator<Item = User>,
    mut reject_file: &File,
) -> Result<(), Box<dyn Error>> {
    let tx = conn.transaction()?;

    {
        let mut insert = tx.prepare_cached(
            "insert or ignore into user (user_id, user_nm, amt) values (?1, ?2, ?3)",
        )?;

        for user in chunk {
            let rows_affected = insert.execute(rusqlite::params![user.user_id, user.user_nm, user.amt])?;

            if rows_affected == 0 {
                let _ = writeln!(
                    reject_file,
                    "INSERT IGNORED: unique constraint violation? [id: {}]",
                    user.user_id
                );
            }
        }
    }

    tx.commit()?;
    Ok(())
}