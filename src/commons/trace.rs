use backtrace::Backtrace;

pub fn trace(mut crop_level: usize, error: Box<dyn std::error::Error>) {
    let mut traces = vec![];
    let bt = Backtrace::new();
    for frame in bt.frames() {
        for symbol in frame.symbols() {
            if let Some(name) = symbol.name() {
                if let Some(filename) = symbol.filename() {
                    let current_dir = std::env::current_dir().unwrap();
                    if filename.starts_with(&current_dir) {
                        let line = symbol.lineno().unwrap_or(0);
                        let filename = filename.strip_prefix(&current_dir).unwrap_or(&filename);
                        traces.push(format!(
                            "\t{}: {}\n\t\t at {}:{}",
                            (traces.len() as i32) - 1,
                            name,
                            filename.display(),
                            line,
                        ))
                    }
                }
            }
        }
    }

    if crop_level > traces.len() {
        crop_level = traces.len();
    }

    eprintln!(
        "{}\n{}",
        error.to_string(),
        traces.split_off(crop_level).join("\n")
    );
}
