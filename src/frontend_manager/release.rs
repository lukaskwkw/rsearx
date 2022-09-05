use anyhow::{anyhow, Result};

#[derive(Clone, Debug, Default)]
pub struct Release {
    pub name: String,
    pub version: String,
    pub date: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}
impl Release {
    pub fn get_first_asset(&self) -> Result<&ReleaseAsset> {
        self.assets
            .first()
            .ok_or_else(|| anyhow!("No assets found"))
    }
}
#[derive(Clone, Debug, Default)]
pub struct ReleaseAsset {
    pub download_url: String,
    pub name: String,
}
pub fn from_release(release: &serde_json::Value) -> Result<Release> {
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("Release missing `tag_name`"))?;
    let date = release["created_at"]
        .as_str()
        .ok_or_else(|| anyhow!("Release missing `created_at`"))?;
    let name = release["name"].as_str().unwrap_or(tag);
    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| anyhow!("No assets found"))?;
    let body = release["body"].as_str().map(String::from);
    let assets = assets
        .iter()
        .map(from_asset)
        .collect::<Result<Vec<ReleaseAsset>>>()?;
    Ok(Release {
        name: name.to_owned(),
        version: tag.trim_start_matches('v').to_owned(),
        date: date.to_owned(),
        body,
        assets,
    })
}
pub fn from_asset(asset: &serde_json::Value) -> Result<ReleaseAsset> {
    let download_url = asset["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Asset missing `url`"))?;
    let name = asset["name"]
        .as_str()
        .ok_or_else(|| anyhow!("Asset missing `name`"))?;
    Ok(ReleaseAsset {
        download_url: download_url.to_owned(),
        name: name.to_owned(),
    })
}
