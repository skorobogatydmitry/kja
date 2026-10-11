use kj_assoc::App;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    ratatui::run(|terminal| App::default().run(terminal))
}
