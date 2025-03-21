use crate::data_locations;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

pub fn backup() {
    let database_location = data_locations().database;
    let output = Command::new("sqlite3")
        .arg(database_location)
        .arg(".dump")
        .output()
        .expect("Failed. Make sure sqlite3 is istalled.");

    print!("{}", String::from_utf8_lossy(&output.stdout));
}

pub fn restore() {
    let database_location = data_locations().database;
    let mut sql_dump = String::new();
    io::stdin()
        .read_to_string(&mut sql_dump)
        .expect("Failed to read input");

    let mut child = Command::new("sqlite3")
        .arg(database_location)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed. Make sure sqlite3 is installed.");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(sql_dump.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child
        .wait_with_output()
        .expect("Failed to wait for child process");

    if !output.status.success() {
        eprintln!(
            "Restore failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("Restore successful");
    }
}
