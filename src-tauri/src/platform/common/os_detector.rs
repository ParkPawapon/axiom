#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OperatingSystem {
    Linux,
    Macos,
    Windows,
    Unknown,
}

pub fn current_os() -> OperatingSystem {
    if cfg!(target_os = "macos") {
        OperatingSystem::Macos
    } else if cfg!(windows) {
        OperatingSystem::Windows
    } else if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else {
        OperatingSystem::Unknown
    }
}
