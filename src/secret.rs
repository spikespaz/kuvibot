use serde::{Deserialize, Serialize};

use twitch_bot::auth;

#[derive(Serialize, Deserialize)]
pub struct TtvConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    // pub db: DbConfig,
    pub ttv: TtvConfig,
}

pub struct Secrets {
    path: std::path::PathBuf,
    pub config: Config,
}

impl Secrets {
    pub fn init_from(path: impl AsRef<std::path::Path>) -> eyre::Result<Self> {
        log::debug!("Initializing secrets");
        let path = path.as_ref().to_owned();
        let config: Config = toml::from_str(&std::fs::read_to_string(path.join("config.toml"))?)?;
        log::debug!("Secrets are all set up");
        Ok(Self { path, config })
    }
    pub fn init() -> eyre::Result<Self> {
        Self::init_from("secret")
    }
    pub async fn get_user_token(
        &self,
        login: impl AsRef<str>,
    ) -> eyre::Result<twitch_oauth2::UserToken> {
        let login = login.as_ref();
        log::debug!("Getting ttv token for {:?}", login);
        let tokens_file_path = self.path.join("tokens").join(format!("{}.json", login));
        let tokens: auth::Tokens = match std::fs::File::open(&tokens_file_path) {
            Ok(file) => {
                log::debug!("Reading existing tokens");
                let tokens: auth::Tokens = serde_json::from_reader(file)?;
                if auth::validate(&tokens.access_token).await? {
                    log::debug!("Token still valid");
                    tokens
                } else {
                    log::debug!("Token invalid, refreshing");
                    auth::refresh(
                        &self.config.ttv.client_id,
                        &self.config.ttv.client_secret,
                        &tokens.refresh_token,
                    )
                    .await?
                }
            }
            Err(_) => {
                log::info!("Auth not setup, prepare to login as {:?}", login);
                auth::authenticate(
                    &self.config.ttv.client_id,
                    &self.config.ttv.client_secret,
                    true,
                    &["channel:read:redemptions", "chat:edit", "chat:read"].map(auth::Scope::new),
                )
                .await?
            }
        };
        std::fs::create_dir_all(tokens_file_path.parent().unwrap())?;
        serde_json::to_writer_pretty(std::fs::File::create(&tokens_file_path)?, &tokens)?;
        log::debug!("Token retrieved successfully");
        Ok(twitch_oauth2::UserToken::from_token(
            &reqwest::Client::new(),
            twitch_oauth2::AccessToken::new(tokens.access_token),
        )
        .await?)
    }
}
