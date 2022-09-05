use std::{
    ffi::OsStr,
    fs::{self},
    path::Path,
};

#[cfg(test)]
use mockall::automock;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{header, Client};
use self_update::Download;
use tokio::task;

use super::{
    release::{from_release, Release},
    unzip::unzip,
};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FEManager {
    async fn fetch_release(&mut self) -> Result<()>;
    async fn download_release(&self) -> Result<()>;
    fn unzip_release(&self) -> Result<()>;
    fn remove_fe_folder(&self) -> Result<()>;
    fn remove_zip_file(&self) -> Result<()>;
    fn does_dir_exists(&self) -> bool;
}

pub struct Manager {
    client: Client,
    repo_owner: String,
    repo_name: String,
    release: Option<Release>,
    fe_path: String,
}

impl Manager {
    fn get_release(&self) -> Result<&Release> {
        self.release
            .as_ref()
            .ok_or_else(|| anyhow!("No release found"))
    }
}

pub struct Executor {
    manager: Box<dyn FEManager>,
}

impl Executor {
    pub fn new(manager: Box<dyn FEManager>) -> Self {
        Self { manager }
    }
    pub fn new_supplied() -> Self {
        let manager = Manager {
            client: Client::new(),
            repo_owner: "lukaskwkw".to_string(),
            repo_name: "rsearx-web".to_string(),
            release: None,
            fe_path: "./web/".to_string(),
        };
        let manager = Box::new(manager);
        Self { manager }
    }
    pub async fn init(&mut self) -> Result<()> {
        self.manager
            .does_dir_exists()
            .then(|| self.manager.remove_fe_folder());
        info!("Fetching release...");
        self.manager.fetch_release().await?;
        info!("Downloading release asset");
        self.manager.download_release().await?;
        info!("Unzipping");
        self.manager.unzip_release()?;
        info!("Removing zip file");
        self.manager.remove_zip_file()?;
        Ok(())
    }
}
#[async_trait]
impl FEManager for Manager {
    async fn fetch_release(&mut self) -> anyhow::Result<()> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/rsearx"
                .parse()
                .expect("github invalid user-agent"),
        );
        let url = format!(
            "{}/repos/{}/{}/releases",
            "https://api.github.com", self.repo_owner, self.repo_name
        );
        let resp = self.client.get(url.clone()).headers(headers).send().await?;
        if !resp.status().is_success() {
            let err_msg = format!(
                "api request failed with status: {:?} - for: {:?}",
                resp.status(),
                url
            );
            return Err(anyhow!(err_msg));
        }

        let releases = resp.json::<serde_json::Value>().await?;
        let releases = releases
            .as_array()
            .ok_or_else(|| anyhow!("No releases found"))?;
        let releases = releases
            .iter()
            .take(1)
            .map(from_release)
            .collect::<Result<Vec<Release>>>()?;
        let release = releases
            .first()
            .ok_or_else(|| anyhow!("No release found"))?;
        self.release = Some(release.clone());
        Ok(())
    }

    async fn download_release(&self) -> Result<()> {
        let url = &self.get_release()?.get_first_asset()?.download_url;
        let mut download = Download::from_url(url);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/rsearx"
                .parse()
                .expect("github invalid user-agent"),
        );
        download.set_headers(headers);
        download.show_progress(true);
        let name = &self.get_release()?.get_first_asset()?.name;
        debug!("Downloading {} to {}", name, name);
        let mut tmp_archive = fs::File::create(&name)
            .map_err(|err| anyhow!("Error during File::create. path {} Err {}", name, err))?;
        task::spawn_blocking(move || {
            download.download_to(&mut tmp_archive).unwrap();
        })
        .await?;
        Ok(())
    }
    fn remove_zip_file(&self) -> Result<()> {
        let name = &self.get_release()?.get_first_asset()?.name;
        fs::remove_file(name)?;
        Ok(())
    }
    fn unzip_release(&self) -> Result<()> {
        let name = &self.get_release()?.get_first_asset()?.name;
        unzip(&name, &self.fe_path, None::<fn(&OsStr) -> bool>)?;
        Ok(())
    }
    fn remove_fe_folder(&self) -> Result<()> {
        fs::remove_dir_all(&self.fe_path)?;
        Ok(())
    }
    fn does_dir_exists(&self) -> bool {
        Path::new(&self.fe_path).exists()
    }
}
#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[actix_rt::test]

    async fn test_init_when_dir_not_exist() {
        let mut mock_manager = MockFEManager::new();
        mock_manager
            .expect_does_dir_exists()
            .return_once_st(|| false)
            .once();
        mock_manager.expect_remove_fe_folder().never();
        mock_manager
            .expect_fetch_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_download_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_unzip_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_remove_zip_file()
            .return_once_st(|| Ok(()))
            .once();
        let mock_manager = Box::new(mock_manager);
        let mut executor = Executor::new(mock_manager);
        assert!(executor.init().await.is_ok());
    }

    #[actix_rt::test]

    async fn test_init_when_dir_exist() {
        let mut mock_manager = MockFEManager::new();

        init_mm_with_basics(&mut mock_manager);

        mock_manager
            .expect_fetch_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_download_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_unzip_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_remove_zip_file()
            .return_once_st(|| Ok(()))
            .once();
        let mock_manager = Box::new(mock_manager);
        let mut executor = Executor::new(mock_manager);

        assert!(executor.init().await.is_ok());
    }

    #[actix_rt::test]

    async fn test_init_but_failed_to_fetch() {
        let mut mock_manager = MockFEManager::new();
        init_mm_with_basics(&mut mock_manager);
        mock_manager
            .expect_fetch_release()
            .return_once_st(|| Err(anyhow!("Failed to fetch")))
            .once();
        let manager = Box::new(mock_manager);
        let mut executor = Executor::new(manager);
        assert!(executor.init().await.is_err());
    }

    #[actix_rt::test]

    async fn test_init_but_failed_to_download() {
        let mut mock_manager = MockFEManager::new();
        init_mm_with_basics(&mut mock_manager);
        mock_manager
            .expect_fetch_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_download_release()
            .return_once_st(|| Err(anyhow!("Failed to download release")))
            .once();
        let mock_manager = Box::new(mock_manager);
        let mut executor = Executor::new(mock_manager);
        assert!(executor.init().await.is_err());
    }

    #[actix_rt::test]

    async fn test_init_but_failed_to_unzip() {
        let mut mock_manager = MockFEManager::new();
        init_mm_with_basics(&mut mock_manager);
        mock_manager
            .expect_fetch_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_download_release()
            .return_once_st(|| Ok(()))
            .once();
        mock_manager
            .expect_unzip_release()
            .return_once_st(|| Err(anyhow!("Failed to unzip release")))
            .once();
        let mock_manager = Box::new(mock_manager);
        let mut executor = Executor::new(mock_manager);
        assert!(executor.init().await.is_err());
    }

    fn init_mm_with_basics(mock_manager: &mut MockFEManager) {
        mock_manager
            .expect_does_dir_exists()
            .return_once_st(|| true)
            .once();
        mock_manager
            .expect_remove_fe_folder()
            .return_once_st(|| Ok(()))
            .once();
    }
}
