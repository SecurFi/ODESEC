use anyhow::{Context, Result};
use ethers_core::types::Chain;
use ethers_providers::{
    is_local_endpoint, Http, HttpRateLimitRetryPolicy, Middleware, Provider, RetryClient,
    RetryClientBuilder, DEFAULT_LOCAL_POLL_INTERVAL,
};
use reqwest::{IntoUrl, Url};
use std::{borrow::Cow, time::Duration};

pub const ALCHEMY_FREE_TIER_CUPS: u64 = 330;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

/// Helper type alias for a retry provider
pub type RetryProvider = Provider<RetryClient<Http>>;

/// Helper type alias for a rpc url
pub type RpcUrl = String;

pub fn get_http_provider(builder: impl Into<ProviderBuilder>) -> RetryProvider {
    try_get_http_provider(builder).unwrap()
}

/// Gives out a provider with a `100ms` interval poll if it's a localhost URL (most likely an anvil
/// or other dev node) and with the default, `7s` if otherwise.
pub fn try_get_http_provider(builder: impl Into<ProviderBuilder>) -> Result<RetryProvider> {
    builder.into().build()
}

/// Helper type to construct a `RetryProvider`
#[derive(Debug)]
pub struct ProviderBuilder {
    // Note: this is a result, so we can easily chain builder calls
    url: Result<Url>,
    chain: Chain,
    max_retry: u32,
    timeout_retry: u32,
    initial_backoff: u64,
    timeout: Duration,
    /// available CUPS
    compute_units_per_second: u64,
}

// === impl ProviderBuilder ===

impl ProviderBuilder {
    /// Creates a new builder instance
    pub fn new(url: impl IntoUrl) -> Self {
        let url_str = url.as_str();
        if url_str.starts_with("localhost:") {
            // invalid url: non-prefixed URL scheme is not allowed, so we prepend the default http
            // prefix
            return Self::new(format!("http://{url_str}"));
        }
        let err = format!("Invalid provider url: {url_str}");
        Self {
            url: url.into_url().context(err),
            chain: Chain::Mainnet,
            max_retry: 100,
            timeout_retry: 5,
            initial_backoff: 100,
            timeout: REQUEST_TIMEOUT,
            // alchemy max cpus <https://github.com/alchemyplatform/alchemy-docs/blob/master/documentation/compute-units.md#rate-limits-cups>
            compute_units_per_second: ALCHEMY_FREE_TIER_CUPS,
        }
    }

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished.
    ///
    /// Default is no timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the chain of the node the provider will connect to
    pub fn chain(mut self, chain: impl Into<Chain>) -> Self {
        self.chain = chain.into();
        self
    }

    /// How often to retry a failed request
    pub fn max_retry(mut self, max_retry: u32) -> Self {
        self.max_retry = max_retry;
        self
    }

    /// How often to retry a failed request due to connection issues
    pub fn timeout_retry(mut self, timeout_retry: u32) -> Self {
        self.timeout_retry = timeout_retry;
        self
    }

    /// The starting backoff delay to use after the first failed request
    pub fn initial_backoff(mut self, initial_backoff: u64) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    /// Sets the number of assumed available compute units per second
    ///
    /// See also, <https://github.com/alchemyplatform/alchemy-docs/blob/master/documentation/compute-units.md#rate-limits-cups>
    pub fn compute_units_per_second(mut self, compute_units_per_second: u64) -> Self {
        self.compute_units_per_second = compute_units_per_second;
        self
    }

    /// Sets aggressive `max_retry` and `initial_backoff` values
    ///
    /// This is only recommend for local dev nodes
    pub fn aggressive(self) -> Self {
        self.max_retry(100).initial_backoff(100)
    }

    /// Same as [`Self:build()`] but also retrieves the `chainId` in order to derive an appropriate
    /// interval
    pub async fn connect(self) -> Result<RetryProvider> {
        let mut provider = self.build()?;
        if let Some(blocktime) = provider.get_chainid().await.ok().and_then(|id| {
            Chain::try_from(id)
                .ok()
                .and_then(|chain| chain.average_blocktime_hint())
        }) {
            provider = provider.interval(blocktime / 2);
        }
        Ok(provider)
    }

    /// Constructs the `RetryProvider` taking all configs into account
    pub fn build(self) -> Result<RetryProvider> {
        let ProviderBuilder {
            url,
            chain,
            max_retry,
            timeout_retry,
            initial_backoff,
            timeout,
            compute_units_per_second,
        } = self;
        let url = url?;

        let client = reqwest::Client::builder().timeout(timeout).build()?;
        let is_local = is_local_endpoint(url.as_str());

        let provider = Http::new_with_client(url, client);

        #[allow(clippy::box_default)]
        let mut provider = Provider::new(
            RetryClientBuilder::default()
                .initial_backoff(Duration::from_millis(initial_backoff))
                .rate_limit_retries(max_retry)
                .timeout_retries(timeout_retry)
                .compute_units_per_second(compute_units_per_second)
                .build(provider, Box::new(HttpRateLimitRetryPolicy::default())),
        );

        if is_local {
            provider = provider.interval(DEFAULT_LOCAL_POLL_INTERVAL);
        } else if let Some(blocktime) = chain.average_blocktime_hint() {
            provider = provider.interval(blocktime / 2);
        }
        Ok(provider)
    }
}

impl<'a> From<&'a str> for ProviderBuilder {
    fn from(url: &'a str) -> Self {
        Self::new(url)
    }
}

impl<'a> From<&'a String> for ProviderBuilder {
    fn from(url: &'a String) -> Self {
        url.as_str().into()
    }
}

impl From<String> for ProviderBuilder {
    fn from(url: String) -> Self {
        url.as_str().into()
    }
}

impl<'a> From<Cow<'a, str>> for ProviderBuilder {
    fn from(url: Cow<'a, str>) -> Self {
        url.as_ref().into()
    }
}
