use anyhow::Result;
use awdy::app::App;
use clap::{Arg, Command};

fn main() -> Result<()> {
    let matches = Command::new("awdy")
        .arg(
            Arg::new("db")
                .short('d')
                .default_value("~/.config/awdy/awdy.db"),
        )
        .get_matches();
    let app = App::load(matches.get_one::<String>("db").unwrap())?;
    app.run()
}
