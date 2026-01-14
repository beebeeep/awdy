use anyhow::Result;
use awdy::app::App;

fn main() -> Result<()> {
    let app = App::load()?;
    app.run()
}
