use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
pub(crate) fn windows_child_process_creation_flags() -> u32 {
    CREATE_NO_WINDOW
}

#[cfg(windows)]
pub fn hide_child_window(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;

    command.creation_flags(windows_child_process_creation_flags())
}

#[cfg(not(windows))]
pub fn hide_child_window(command: &mut Command) -> &mut Command {
    command
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn windows_child_processes_use_no_window_creation_flag() {
        assert_eq!(super::windows_child_process_creation_flags(), 0x08000000);
    }
}
