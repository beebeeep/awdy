use std::fs;

use anyhow::{Result, anyhow};
use awdy::app::App;
use clap::{Arg, Command};

fn main() -> Result<()> {
    let matches = Command::new("awdy")
        .arg(Arg::new("db").short('d'))
        .get_matches();
    let db_file = match matches.get_one::<String>("db") {
        Some(f) => f,
        None => {
            if fs::exists(".awdy.db")? {
                ".awdy.db"
            } else if fs::exists("~/.config/awdy/awdy.db")? {
                "~/.config/awdy/awdy.db"
            } else {
                return Err(anyhow!(
                    "Cannot find database in neiter in .awdy.db, nor in ~/.config/awdy/awdy.db, specify DB location using -d argument"
                ));
            }
        }
    };
    let app = App::load(db_file)?;
    app.run()
}
