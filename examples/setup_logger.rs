pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%+"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::fs::File::create("output.log")?)
        .apply()?;
    Ok(())
}
