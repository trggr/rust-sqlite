use csv::ReaderBuilder;
use itertools::Itertools;
use serde::Deserialize;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Debug, Deserialize)]
struct RawRecord {
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = rusqlite::Connection::open("sqlite.db")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "OFF")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            user_id TEXT PRIMARY KEY,
            user_nm TEXT NOT NULL,
            amt INTEGER NOT NULL
        )",
        [],
    )?;

    let reject_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rejected_users.txt")?;

    ReaderBuilder::new()
        .has_headers(true)
        .from_path("users.csv")?
        .into_deserialize::<RawRecord>()
        .map(|res| res.map_err(|e| e.to_string()))
        .map(|res| {
            res.and_then(|raw| match raw.amt {
                Some(amt) => Ok(User {
                    user_id: raw.user_id,
                    user_nm: raw.user_nm,
                    amt,
                }),
                None => Err("Missing or invalid 'amt' value".to_string()),
            })
        })
        .filter_map(|res| {
            res.inspect_err(|err| {
                let _ = writeln!(&reject_file, "REJECTED: {}", err);
            })
            .ok()
        })
        .chunks(10_000)
        .into_iter()
        .fold(Ok(()), |acc, chunk| {
            acc.and_then(|_| process_transaction_chunk(&mut conn, chunk, &reject_file))
        })
}

fn process_transaction_chunk(
    conn: &mut rusqlite::Connection,
    chunk: impl Iterator<Item = User>,
    reject_file: &File,
) -> Result<(), Box<dyn Error>> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "insert into user (user_id, user_nm, amt) values (?1, ?2, ?3)",
        )?;

        chunk.for_each(|user| {
            stmt.execute(rusqlite::params![user.user_id, user.user_nm, user.amt])
                .map(|_| ())
                .inspect_err(|err| {
                    let mut writer = reject_file;
                    let _ = writeln!(
                        writer,
                        "DB INSERT FAILED [id: {}]: {}",
                        user.user_id, err
                    );
                })
                .unwrap_or(());
        });
    }
    tx.commit()?;
    Ok(())
}