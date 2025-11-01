use common::dtos::{LangDto, TaggedGroupDto};
use reqwest::Url;
use rocket::{
    fairing::{self, Fairing},
    Build, Rocket,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiveConfig {
    pub url: Url,
    pub secret: String,
}

pub struct HiveClient {
    client: reqwest::Client,
    config: HiveConfig,
}

impl HiveClient {
    pub fn new(config: HiveConfig) -> reqwest::Result<Self> {
        let client = reqwest::Client::builder().build()?;

        Ok(Self { client, config })
    }

    /// Get group memberships for the user with the specified username that have been tagged with
    /// META TV's tag.
    ///
    /// On a high level this can be thought of as returning the group memberships for the user that
    /// this application is allowed to see.
    ///
    /// Wrapper around `/tagged/slide-manager/memberships/{username}`.
    pub async fn tagged_memberships(
        &self,
        username: &str,
        lang: LangDto,
    ) -> reqwest::Result<Vec<TaggedGroupDto>> {
        let mut url = self.config.url.clone();
        url.path_segments_mut()
            .expect("url can be a base")
            .pop_if_empty()
            .extend(&["tagged", "slide-manager", "memberships", username]);
        url.query_pairs_mut()
            .append_pair("lang", &format!("{}", lang));
        self.client
            .get(url)
            .bearer_auth(&self.config.secret)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<TaggedGroupDto>>()
            .await
    }

    /// Get all groups that have been tagged with META TV's tag.
    ///
    /// Wrapper around `/tagged/slide-manager/groups`.
    pub async fn tagged_groups(&self, lang: LangDto) -> reqwest::Result<Vec<TaggedGroupDto>> {
        let mut url = self.config.url.clone();
        url.path_segments_mut()
            .expect("url can be a base")
            .pop_if_empty()
            .extend(&["tagged", "slide-manager", "groups"]);
        url.query_pairs_mut()
            .append_pair("lang", &format!("{}", lang));
        self.client
            .get(url)
            .bearer_auth(&self.config.secret)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<TaggedGroupDto>>()
            .await
    }
}

pub struct HiveInitializer;

#[rocket::async_trait]
impl Fairing for HiveInitializer {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Hive Client",
            kind: fairing::Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let config = match rocket.figment().focus("hive").extract::<HiveConfig>() {
            Ok(config) => config,
            Err(e) => {
                error!("hive configuration incomplete: {}", e);
                return Err(rocket);
            }
        };

        let client = match HiveClient::new(config) {
            Ok(client) => client,
            Err(e) => {
                error!("failed to initialize hive client: {}", e);
                return Err(rocket);
            }
        };

        Ok(rocket.manage(client))
    }
}
