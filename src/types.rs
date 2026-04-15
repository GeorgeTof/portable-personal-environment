#[derive(Debug, PartialEq, Eq)]
pub enum DistroFamily {
    Debian,
    RedHat,
    Arch,
    OpenSuse,
    Unknown(String),
}

impl DistroFamily {
    pub fn is_supported(&self) -> bool {
        matches!(self, DistroFamily::Debian)
    }

    pub fn from_os_string(family_string: &str) -> Self {
        let s = family_string.to_lowercase();
        
        if s.contains("debian") {
            DistroFamily::Debian
        } else if s.contains("fedora") || s.contains("rhel") {
            DistroFamily::RedHat
        } else if s.contains("arch") {
            DistroFamily::Arch
        } else if s.contains("suse") {
            DistroFamily::OpenSuse
        } else {
            DistroFamily::Unknown(s)
        }
    }
}