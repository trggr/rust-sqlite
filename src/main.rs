use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;
use std::fs::OpenOptions;
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
    // 1. Configure SQLite for raw write speed
    let mut conn = rusqlite::Connection::open("sqlite.db")?;
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

    let mut reject_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rejected_users.txt")?;

    // 2. Stream & transform lazily via owned Iterator combinators
    let records = ReaderBuilder::new()
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
        });

    // 3. Process in transactional batches (10k items per batch for peak SQLite throughput)
    let batch_size = 10_000;
    let mut current_batch = Vec::with_capacity(batch_size);

    for result in records {
        let _ = result
            .inspect(|user| current_batch.push(User {
                user_id: user.user_id.clone(),
                user_nm: user.user_nm.clone(),
                amt: user.amt,
            }))
            .inspect_err(|err| {
                let _ = writeln!(reject_file, "REJECTED: {}", err);
            });

        if current_batch.len() >= batch_size {
            flush_batch(&mut conn, &mut current_batch, &mut reject_file)?;
        }
    }

    // Flush remaining records
    if !current_batch.is_empty() {
        flush_batch(&mut conn, &mut current_batch, &mut reject_file)?;
    }

    Ok(())
}

fn flush_batch(
    conn: &mut rusqlite::Connection,
    batch: &mut Vec<User>,
    reject_file: &mut std::fs::File,
) -> Result<(), Box<dyn Error>> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "insert into user (user_id, user_nm, amt) values (:1, :2, :3)",
        )?;

        batch.drain(..)
            .for_each(|user| {
                stmt.execute(rusqlite::params![user.user_id, user.user_nm, user.amt])
                .map(|_| ())
                .inspect_err(|err| {
                    let _ = writeln!(
                        reject_file,
                        "DB INSERT FAILED [id: {}]: {}",
                        user.user_id, err
                    );
                })
                .or_else(|_| Ok::<(), rusqlite::Error>(()))
                .unwrap();
        });
    }
    tx.commit()?;
    Ok(())
}
