use std::io;
use std::process::{self, Command};

/// Conveniance function to convert an io result into a `Result<String, String>`.
fn convert(output: io::Result<process::Output>) -> Result<String, String> {
    match output {
        Ok(o) => {
            if o.status.success() {
                Ok(format!(
                    "Success.\nstatus: {}\nstdout: {}\nstderr: {}",
                    o.status,
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                ))
            } else {
                Err(format!(
                    "ERROR!\nstatus: {}\nstdout: {}\nstderr: {}",
                    o.status,
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                ))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Adds a right click context menu entry for `papyrus`.
/// Associates `.rs` and `.rscript` files with `papyrus`.
///
/// # Windows
/// This involves changes to the registry and will require elevated permissions.
///
/// # Linux and Mac
/// Currently not implemented and will result in undefined behaviour.
pub fn add_right_click_menu() -> Result<String, String> {
    self::windows::add_windows_right_click_menu()
}

/// Removes the right click context menu entry for `papyrus`.
///
/// # Windows
/// This involves changes to the registry and will require elevated permissions.
///
/// # Linux and Mac
/// Currently not implemented and will result in undefined behaviour.
pub fn remove_right_click_menu() -> Result<String, String> {
    self::windows::remove_windows_right_click_menu()
}

mod windows {
    use super::*;
    use std::env;

    pub fn add_windows_right_click_menu() -> Result<String, String> {
        let path_to_exe = env::current_exe().map_err(|_| "failed to load exe path".to_string())?;

        // add the .rs entry
        convert(
            Command::new("reg")
                .arg("add")
                .arg("HKCR\\.rs")
                .args(&["/d", "rustsrcfile", "/f"])
                .output(),
        )?;
        // add the .rscript entry
        convert(
            Command::new("reg")
                .arg("add")
                .arg("HKCR\\.rscript")
                .args(&["/d", "rustsrcfile", "/f"])
                .output(),
        )?;
        // add the shell menu
        convert(
            Command::new("reg")
                .arg("add")
                .arg("HKCU\\Software\\Classes\\rustsrcfile\\shell\\Run with Papyrus\\command")
                .args(&[
                    "/d",
                    format!("{:?} \"run\" \"%1\"", path_to_exe.as_os_str()).as_str(),
                    "/f",
                ])
                .output(),
        )?;

        Ok("commands successfuly executed".to_string())
    }

    pub fn remove_windows_right_click_menu() -> Result<String, String> {
        // add the .rs entry
        convert(
            Command::new("reg")
                .arg("delete")
                .arg("HKCR\\.rs")
                .arg("/f")
                .output(),
        )?;
        // add the .rscript entry
        convert(
            Command::new("reg")
                .arg("delete")
                .arg("HKCR\\.rscript")
                .arg("/f")
                .output(),
        )?;
        // add the shell menu
        convert(
            Command::new("reg")
                .arg("delete")
                .arg("HKCU\\Software\\Classes\\rustsrcfile")
                .arg("/f")
                .output(),
        )?;

        Ok("commands successfully executed".to_string())
    }
}
