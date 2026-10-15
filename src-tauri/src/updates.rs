//! Update policy is independent of the desktop transport and network contexts.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
}
impl Channel {
    pub fn feed(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta-public",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }
    pub fn saved(value: &str, version: &str) -> Self {
        match value {
            "stable" => Self::Stable,
            "beta" => Self::Beta,
            _ if version.contains('-') => Self::Beta,
            _ => Self::Stable,
        }
    }
    pub fn accepts(self, version: &str) -> bool {
        semver::Version::parse(version).is_ok_and(|v| self == Self::Beta || v.pre.is_empty())
    }
}
pub fn due(now: u64, last_success: u64, last_attempt: u64) -> bool {
    (last_success == 0 || now.saturating_sub(last_success) >= 86_400)
        && (last_attempt == 0 || now.saturating_sub(last_attempt) >= 3_600)
}
pub fn download_url(version: &str) -> Option<String> {
    let v = semver::Version::parse(version).ok()?;
    if v.to_string() != version {
        return None;
    }
    let (platform, extension) = if cfg!(target_os = "macos") {
        ("macos-arm64", ".app.tar.gz")
    } else if cfg!(target_os = "windows") {
        ("windows-x64", ".exe")
    } else {
        ("linux-x64", ".AppImage")
    };
    Some(format!(
        "https://github.com/youssefsz/local-chat/releases/download/v{version}/Local-Chat_{version}_{platform}{extension}"
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_policy_and_daily_backoff() {
        assert!(Channel::Beta.accepts("0.1.0-beta.7"));
        assert!(Channel::Beta.accepts("0.1.0"));
        assert!(!Channel::Stable.accepts("0.1.0-beta.7"));
        assert!(!Channel::Beta.accepts("invalid"));
        assert_eq!(Channel::saved("", "0.1.0-beta.6"), Channel::Beta);
        assert_eq!(Channel::saved("", "0.1.0"), Channel::Stable);
        assert_eq!(Channel::Beta.feed(), "beta-public");
        assert_eq!(Channel::Stable.feed(), "stable");
        assert!(due(10, 0, 0));
        assert!(!due(100, 0, 90));
        assert!(!due(86_500, 86_000, 0));
        assert!(due(90_000, 1, 86_000));
        assert!(!due(1, 2, 2));
    }
    #[test]
    fn update_downloads_are_bound_to_our_release_and_version() {
        assert!(
            download_url("0.1.0-beta.6")
                .unwrap()
                .contains("/v0.1.0-beta.6/Local-Chat_0.1.0-beta.6_")
        );
        assert!(download_url("../other").is_none());
        assert!(download_url("0.01.0").is_none());
    }
}
