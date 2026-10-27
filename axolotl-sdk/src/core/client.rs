use crate::core::http::HttpClientExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;

pub struct NoAuthMode;
pub struct AuthMode;

pub struct MCLClient<Mode> {
    pub(crate) mode: Mode,
    pub(crate) client: Client,
    pub(crate) limiter: Arc<Semaphore>,
    pub(crate) token: Option<String>,

    #[cfg(feature = "piston")]
    pub piston: PistonClient,

    #[cfg(feature = "mojang")]
    pub mojang: MojangClient,

    #[cfg(feature = "forge")]
    pub forge: ForgeClient,

    #[cfg(feature = "fabric")]
    pub fabric: FabricClient,

    #[cfg(feature = "neoforge")]
    pub neoforge: NeoForgeClient,

    #[cfg(feature = "quilt")]
    pub quilt: QuiltClient,
}

impl MCLClient<NoAuthMode> {
    pub fn new(pool_size: usize) -> MCLClient<NoAuthMode> {
        Self::builder().pool_size(pool_size).build()
    }

    pub fn builder() -> MCLClientBuilder<NoAuthMode> {
        MCLClientBuilder::new()
    }

    pub fn authentication(self, token: String) -> MCLClient<AuthMode> {
        Self::builder()
            .client(self.client)
            .limiter(self.limiter)
            .authentication(token).build()
    }
}

impl MCLClient<AuthMode> {
    pub fn token(&self) -> &str {
        self.token.as_deref().unwrap_or("")
    }
}

#[cfg(feature = "piston")]
pub struct PistonClient {
    pub(crate) base_url: Url,
    pub(crate) client: Client,
    pub(crate) limiter: Arc<Semaphore>,
}

#[cfg(feature = "piston")]
impl PistonClient {
    pub fn new(client: Client, limiter: Arc<Semaphore>) -> Self {
        Self {
            base_url: Url::parse("https://piston-meta.mojang.com/").unwrap(),
            client,
            limiter,
        }
    }
}

#[cfg(feature = "piston")]
impl HttpClientExt for PistonClient {
    fn client(&self) -> &Client {
        &self.client
    }

    fn limiter(&self) -> &Semaphore {
        &self.limiter
    }
}

#[cfg(feature = "mojang")]
pub struct MojangClient {}

#[cfg(feature = "forge")]
pub struct ForgeClient {}

#[cfg(feature = "fabric")]
pub struct FabricClient {
    pub(crate) base_url: Url,
    pub(crate) client: Client,
    pub(crate) limiter: Arc<Semaphore>,
}

#[cfg(feature = "fabric")]
impl HttpClientExt for FabricClient {
    fn client(&self) -> &Client {
        &self.client
    }

    fn limiter(&self) -> &Semaphore {
        &self.limiter
    }
}

#[cfg(feature = "fabric")]
impl FabricClient {
    pub fn new(client: Client, limiter: Arc<Semaphore>) -> Self {
        Self {
            base_url: Url::parse("https://meta.fabricmc.net/").unwrap(),
            client,
            limiter,
        }
    }
}

#[cfg(feature = "neoforge")]
pub struct NeoForgeClient {}

#[cfg(feature = "quilt")]
pub struct QuiltClient {}

pub struct MCLClientBuilder<Mode> {
    mode: Mode,
    custom_client: Option<Client>,
    user_agent: String,
    custom_limiter: Option<Arc<Semaphore>>,
    pool_size: usize,
    token: Option<String>,
}

impl<Mode> MCLClientBuilder<Mode> {
    pub fn client(mut self, client: Client) -> Self {
        self.custom_client = Some(client);
        self
    }

    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn limiter(mut self, limiter: Arc<Semaphore>) -> Self {
        self.custom_limiter = Some(limiter);
        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }

    pub fn build(self) -> MCLClient<Mode> {
        let client = match self.custom_client {
            Some(c) => c,
            None => Client::builder()
                .user_agent(self.user_agent)
                .build()
                .unwrap(),
        };

        let limiter = match self.custom_limiter {
            Some(l) => l,
            None => Arc::new(Semaphore::new(self.pool_size)),
        };

        MCLClient {
            mode: self.mode,
            client: client.clone(),
            limiter: limiter.clone(),
            token: self.token,
            #[cfg(feature = "piston")]
            piston: PistonClient::new(client.clone(), limiter.clone()),
            #[cfg(feature = "mojang")]
            mojang: MojangClient {},
            #[cfg(feature = "forge")]
            forge: ForgeClient {},
            #[cfg(feature = "fabric")]
            fabric: FabricClient::new(client.clone(), limiter.clone()),
            #[cfg(feature = "neoforge")]
            neoforge: NeoForgeClient {},
            #[cfg(feature = "quilt")]
            quilt: QuiltClient {},
        }
    }
}

impl MCLClientBuilder<NoAuthMode> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn authentication(self, token: String) -> MCLClientBuilder<AuthMode> {
        MCLClientBuilder::<AuthMode> {
            mode: AuthMode,
            custom_client: self.custom_client,
            user_agent: self.user_agent,
            custom_limiter: self.custom_limiter,
            pool_size: self.pool_size,
            token: Some(token),
        }
    }
}

impl Default for MCLClientBuilder<NoAuthMode> {
    fn default() -> Self {
        Self {
            mode: NoAuthMode,
            custom_client: None,
            user_agent: "AxolotlCDK/1.0".to_string(),
            custom_limiter: None,
            pool_size: 8,
            token: None,
        }
    }
}
