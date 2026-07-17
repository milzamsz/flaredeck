use crate::cloudflared::flaredeck_index_path;
use crate::error::AppResult;
use crate::types::ProfileIndex;

pub fn parse_index(raw: &str) -> ProfileIndex {
    serde_json::from_str(raw).unwrap_or_default()
}

/// Safe profile-index read shared by desktop and headless interfaces.
pub async fn list() -> AppResult<ProfileIndex> {
    let path = flaredeck_index_path()?;
    if !path.exists() {
        return Ok(ProfileIndex::default());
    }
    Ok(parse_index(&tokio::fs::read_to_string(path).await?))
}

#[cfg(test)]
mod tests {
    use super::parse_index;

    #[test]
    fn invalid_index_is_empty_and_contains_no_token_data() {
        let index = parse_index("not-json");
        assert!(index.profiles.is_empty());
        assert_eq!(index.active_profile_id, None);
    }
}
