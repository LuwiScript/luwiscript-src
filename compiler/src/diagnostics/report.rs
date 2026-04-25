use crate::diagnostics::error::{Diagnostic, Diagnostics, Level};

pub fn report_diagnostics(diagnostics: &Diagnostics, source: &str, file_name: &str) {
    for diag in diagnostics.items() {
        report_one(diag, source, file_name);
    }
}

fn report_one(diag: &Diagnostic, source: &str, file_name: &str) {
    let level_str = match diag.level {
        Level::Error => "\x1b[31merror\x1b[0m",
        Level::Warning => "\x1b[33mwarning\x1b[0m",
        Level::Note => "\x1b[36mnote\x1b[0m",
    };

    let code_str = diag.code.as_ref().map(|c| format!("[{c}]")).unwrap_or_default();

    eprintln!("{level_str}{code_str}: {}", diag.message);

    if let Some(span) = diag.span {
        let line_idx = source[..span.start].matches('\n').count();
        let lines: Vec<&str> = source.lines().collect();
        if let Some(line) = lines.get(line_idx) {
            eprintln!("  --> {file_name}:{}:{}", span.line, span.col);
            eprintln!("   |");
            eprintln!("{} | {line}", line_idx + 1);
            eprint!("   | ");
            for _ in 0..span.col.saturating_sub(1) {
                eprint!(" ");
            }
            let len = if span.end > span.start { span.end - span.start } else { 1 };
            for _ in 0..len.max(1) {
                eprint!("^");
            }
            eprintln!();
        }
    }

    for hint in &diag.hints {
        eprintln!("  = hint: {hint}");
    }
}
