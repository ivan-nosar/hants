use arboard::Clipboard;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub enum OutputTarget {
    Console,
    Clipboard,
    File(PathBuf),
}

pub fn parse_output_option(s: &str) -> Result<OutputTarget, String> {
    match s.to_lowercase().as_str() {
        "c" | "console" => Ok(OutputTarget::Console),
        "cb" | "clipboard" => Ok(OutputTarget::Clipboard),
        other => try_parse_file_output_option(other),
    }
}

// TODO: Consider optimization for large output that might not fit memory (use streams?)
pub fn write_output(target: OutputTarget, content: String) -> Result<(), String> {
    match target {
        OutputTarget::Console => println!("{}", content),
        OutputTarget::Clipboard => {
            let mut clipboard = Clipboard::new().map_err(|err| err.to_string())?;
            clipboard.set_text(content).map_err(|e| e.to_string())?;
        }
        OutputTarget::File(path) => {
            if path.exists() {
                return Err(format!("file already exists: {}", path.display()));
            }
            fs::write(&path, &content).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Checks if option value passed is a valid file path.
/// Returns `File(PathBuf)` instance if argument represent a valid file path
/// and the file does not exist. Otherwise, returns `Err(String)` with the error message.
fn try_parse_file_output_option(option_value: &str) -> Result<OutputTarget, String> {
    let trimmed = option_value.trim();
    if trimmed.is_empty() {
        return Err(invalid_output_option_message(option_value));
    }

    let path = PathBuf::from(trimmed);

    // Reject path-like strings that the host OS would refuse. On Windows this catches
    // reserved characters (<>:"|?*) and similar issues surfaced by metadata lookups.
    if let Err(e) = path.canonicalize() {
        if e.kind() == std::io::ErrorKind::NotFound {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                return Err(invalid_output_option_message(option_value));
            }
            return Ok(OutputTarget::File(path));
        }
        return Err(invalid_output_option_message(option_value));
    }

    Err(format!(
        "file or directory already exists: '{}'",
        path.display()
    ))
}

fn invalid_output_option_message(option_value: &str) -> String {
    format!(
        "unexpected output option '{}' found. Supported options are 'c', 'console', 'cb', \
        'clipboard', or a valid path to a non-existent file.",
        option_value
    )
}
