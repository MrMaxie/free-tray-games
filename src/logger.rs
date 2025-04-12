use anyhow::Result;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

pub fn init_logger() -> Result<()> {
    let log_writer = {
        let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
        let log_path = exe_dir.join("error.log");

        BoxMakeWriter::new(move || {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Failed to open error.log")
        })
    };

    tracing_subscriber::fmt()
        .with_writer(log_writer)
        .with_ansi(false)
        .with_max_level(tracing::Level::ERROR)
        .with_target(false)
        .with_line_number(true)
        .with_file(true)
        .init();

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("Panic occurred: {:?}", info);
    }));

    Ok(())
}