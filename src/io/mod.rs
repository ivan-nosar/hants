use arboard::Clipboard;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub enum IoTarget {
    Console,
    Clipboard,
    File(PathBuf),
}

pub fn parse_output_option(s: &str) -> Result<IoTarget, String> {
    parse_io_option(s, IoDirection::Output)
}

pub fn parse_input_option(s: &str) -> Result<IoTarget, String> {
    parse_io_option(s, IoDirection::Input)
}

// TODO: Consider optimization for large output that might not fit memory (use streams?)
pub fn write_output(target: IoTarget, content: String) -> Result<(), String> {
    match target {
        IoTarget::Console => println!("{}", content),
        IoTarget::Clipboard => {
            let mut clipboard = Clipboard::new().map_err(|err| err.to_string())?;
            clipboard.set_text(content).map_err(|e| e.to_string())?;
        }
        IoTarget::File(path) => {
            if path.exists() {
                return Err(format!("file already exists: {}", path.display()));
            }
            fs::write(&path, &content).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn parse_io_option(s: &str, direction: IoDirection) -> Result<IoTarget, String> {
    match s.to_lowercase().as_str() {
        "c" | "console" => Ok(IoTarget::Console),
        "cb" | "clipboard" => Ok(IoTarget::Clipboard),
        other => try_parse_file_option(other, direction),
    }
}

/// Checks if option value passed is a valid file path.
/// Returns `File(PathBuf)` instance if argument represent a valid file path
/// and the file does not exist. Otherwise, returns `Err(String)` with the error message.
fn try_parse_file_option(option_value: &str, direction: IoDirection) -> Result<IoTarget, String> {
    let trimmed = option_value.trim();
    if trimmed.is_empty() {
        return Err(invalid_io_option_message(option_value, direction));
    }

    let path = PathBuf::from(trimmed);

    // Reject path-like strings that the host OS would refuse. On Windows this catches
    // reserved characters (<>:"|?*) and similar issues surfaced by metadata lookups.
    let path_canonicalization_result = path.canonicalize();

    match direction {
        IoDirection::Input => {
            if let Err(_) = path_canonicalization_result {
                // Return error on any error kind:
                // - io::ErrorKind::NotFound: when file is not found
                // - io::ErrorKind::InvalidInput: the file or a component of the path does not exist
                // - io::ErrorKind::PermissionDenied: program lacks the necessary read permissions
                return Err(invalid_io_option_message(option_value, direction));
            }

            // `metadata` call will return error if filesystem entity doesn't exist
            match fs::metadata(&path) {
                Err(_) => Err(invalid_io_option_message(option_value, direction)),
                Ok(metadata) => {
                    if metadata.is_file() {
                        Ok(IoTarget::File(path_canonicalization_result.unwrap()))
                    } else {
                        Err(invalid_io_option_message(option_value, direction))
                    }
                },
            }
        }
        IoDirection::Output => {
            if let Err(e) = path_canonicalization_result {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // TODO: Consider creating parent directories recursively
                    // Ensure parent directory for the new file exists. Return error if not.
                    if let Some(parent) = path.parent()
                        && !parent.as_os_str().is_empty()
                        && !parent.exists()
                    {
                        return Err(invalid_io_option_message(option_value, direction));
                    }

                    return Ok(IoTarget::File(path));
                }

                // Return error if any other issue was found on canonicalization,
                // such as lack of permissions or invalid path components.
                return Err(invalid_io_option_message(option_value, direction));
            }

            Err(format!(
                "file or directory already exists: '{}'",
                path.display()
            ))
        }
    }
}

fn invalid_io_option_message(option_value: &str, direction: IoDirection) -> String {
    let file_comment = match direction {
        IoDirection::Input => "an existing",
        IoDirection::Output => "a non-existent",
    };

    format!(
        "unexpected {} option '{}' found. Supported options are 'c', 'console', 'cb', \
        'clipboard', or a valid path to {} file.",
        direction, option_value, file_comment
    )
}

enum IoDirection {
    Input,
    Output,
}

impl std::fmt::Display for IoDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoDirection::Input => write!(f, "input"),
            IoDirection::Output => write!(f, "output"),
        }
    }
}
