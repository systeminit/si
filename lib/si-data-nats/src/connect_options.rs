use async_nats::{Auth, AuthError, ToServerAddrs};
use futures::Future;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use super::{Client, Result};

/// Connect options. Used to connect with NATS when custom config is needed.
/// # Examples
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), si_data_nats::Error> {
/// let mut options = si_data_nats::ConnectOptions::new()
///     .require_tls(true)
///     .ping_interval(std::time::Duration::from_secs(10))
///     .connect("demo.nats.io")
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ConnectOptions {
    pub(crate) inner: async_nats::ConnectOptions,
}

impl ConnectOptions {
    /// Enables customization of NATS connection.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let mut options = si_data_nats::ConnectOptions::new()
    ///     .require_tls(true)
    ///     .ping_interval(std::time::Duration::from_secs(10))
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Connect to the NATS Server leveraging all passed options.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .require_tls(true)
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Pass multiple URLs.
    ///
    /// ```no_run
    /// #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// use si_data_nats::ServerAddr;
    /// let client = si_data_nats::connect(vec![
    ///     "demo.nats.io".parse::<ServerAddr>()?,
    ///     "other.nats.io".parse::<ServerAddr>()?,
    /// ])
    /// .await
    /// .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(
        self,
        addrs: impl ToServerAddrs,
        subject_prefix: Option<String>,
    ) -> Result<Client> {
        Client::connect_with_options(addrs, subject_prefix, self).await
    }

    /// Creates a builder with a custom auth callback to be used when authenticating against the NATS Server.
    /// Requires an asynchronous function that accepts nonce and returns [Auth].
    /// It will overwrite all other auth methods used.
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::ConnectError> {
    /// si_data_nats::ConnectOptions::with_auth_callback(move |_| async move {
    ///     let mut auth = async_nats::Auth::new();
    ///     auth.username = Some("derek".to_string());
    ///     auth.password = Some("s3cr3t".to_string());
    ///     Ok(auth)
    /// })
    /// .connect("demo.nats.io")
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_auth_callback<F, Fut>(callback: F) -> Self
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<Auth, AuthError>> + 'static + Send + Sync,
    {
        async_nats::ConnectOptions::with_auth_callback(callback).into()
    }

    /// Authenticate against NATS Server with the provided token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::with_token("t0k3n!".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_token(token: String) -> Self {
        async_nats::ConnectOptions::with_token(token).into()
    }

    /// Use a builder to specify a token, to be used when authenticating against the NATS Server.
    /// This can be used as a way to mix authentication methods.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .token("t0k3n!".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn token(self, token: String) -> Self {
        self.inner.token(token).into()
    }

    /// Authenticate against NATS Server with the provided username and password.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::with_user_and_password("derek".into(), "s3cr3t!".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_user_and_password(user: String, password: String) -> Self {
        async_nats::ConnectOptions::with_user_and_password(user, password).into()
    }

    /// Use a builder to specify a username and password, to be used when authenticating against
    /// the NATS Server. This can be used as a way to mix authentication methods.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .user_and_password("derek".into(), "s3cr3t!".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_and_password(self, user: String, pass: String) -> Self {
        self.inner.user_and_password(user, pass).into()
    }

    /// Authenticate with an NKey. Requires an NKey Seed secret.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let nc = si_data_nats::ConnectOptions::with_nkey(seed.into())
    ///     .connect("localhost")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_nkey(seed: String) -> Self {
        async_nats::ConnectOptions::with_nkey(seed).into()
    }

    /// Use a builder to specify an NKey, to be used when authenticating against the NATS Server.
    /// Requires an NKey Seed Secret. This can be used as a way to mix authentication methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .nkey(seed.into())
    ///     .connect("localhost")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn nkey(self, seed: String) -> Self {
        self.inner.nkey(seed).into()
    }

    /// Authenticate with a JWT. Requires function to sign the server nonce. The signing function
    /// is asynchronous.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::ConnectError> {
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let key_pair = std::sync::Arc::new(nkeys::KeyPair::from_seed(seed).unwrap());
    /// // load jwt from creds file or other secure source
    /// async fn load_jwt() -> std::io::Result<String> {
    ///     todo!();
    /// }
    /// let jwt = load_jwt().await?;
    /// let nc = si_data_nats::ConnectOptions::with_jwt(jwt, move |nonce| {
    ///     let key_pair = key_pair.clone();
    ///     async move { key_pair.sign(&nonce).map_err(async_nats::AuthError::new) }
    /// })
    /// .connect("localhost")
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_jwt<F, Fut>(jwt: String, sign_cb: F) -> Self
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<Vec<u8>, AuthError>> + 'static + Send + Sync,
    {
        async_nats::ConnectOptions::with_jwt(jwt, sign_cb).into()
    }

    /// Use a builder to specify a JWT, to be used when authenticating against the NATS Server.
    /// Requires an asynchronous function to sign the server nonce. This can be used as a way to
    /// mix authentication methods.
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::ConnectError> {
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let key_pair = std::sync::Arc::new(nkeys::KeyPair::from_seed(seed).unwrap());
    /// // load jwt from creds file or other secure source
    /// async fn load_jwt() -> std::io::Result<String> {
    ///     todo!();
    /// }
    /// let jwt = load_jwt().await?;
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .jwt(jwt, move |nonce| {
    ///         let key_pair = key_pair.clone();
    ///         async move { key_pair.sign(&nonce).map_err(async_nats::AuthError::new) }
    ///     })
    ///     .connect("localhost")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn jwt<F, Fut>(self, jwt: String, sign_cb: F) -> Self
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<Vec<u8>, AuthError>> + 'static + Send + Sync,
    {
        self.inner.jwt(jwt, sign_cb).into()
    }

    /// Authenticate with NATS using a `.creds` file. Open the provided file, load its creds, and
    /// perform the desired authentication
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::with_credentials_file("path/to/my.creds")
    ///     .await?
    ///     .connect("connect.ngs.global")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_credentials_file(path: impl AsRef<Path>) -> Result<Self> {
        Ok(async_nats::ConnectOptions::with_credentials_file(path)
            .await?
            .into())
    }

    /// Use a builder to authenticate with NATS using a `.creds` file. Open the provided file, load
    /// its creds, and perform the desired authentication
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .credentials("path/to/my.creds")
    ///     .await?
    ///     .connect("connect.ngs.global")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn credentials_file(self, path: impl AsRef<Path>) -> Result<Self> {
        Ok(self.inner.credentials_file(path).await?.into())
    }

    /// Authenticate with NATS using a credential str, in the creds file format.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let creds = "-----BEGIN NATS USER JWT-----
    /// eyJ0eXAiOiJqd3QiLCJhbGciOiJlZDI1NTE5...
    /// ------END NATS USER JWT------
    ///
    /// ************************* IMPORTANT *************************
    /// NKEY Seed printed below can be used sign and prove identity.
    /// NKEYs are sensitive and should be treated as secrets.
    ///
    /// -----BEGIN USER NKEY SEED-----
    /// SUAIO3FHUX5PNV2LQIIP7TZ3N4L7TX3W53MQGEIVYFIGA635OZCKEYHFLM
    /// ------END USER NKEY SEED------
    /// ";
    ///
    /// let nc = si_data_nats::ConnectOptions::with_credentials(creds)
    ///     .expect("failed to parse static creds")
    ///     .connect("connect.ngs.global")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_credentials(creds: &str) -> Result<Self> {
        Ok(async_nats::ConnectOptions::with_credentials(creds)?.into())
    }

    /// Use a builder to specify a credentials string, to be used when authenticating against the
    /// NATS Server. The string should be in the credentials file format. This can be used as a way
    /// to mix authentication methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let creds = "-----BEGIN NATS USER JWT-----
    /// eyJ0eXAiOiJqd3QiLCJhbGciOiJlZDI1NTE5...
    /// ------END NATS USER JWT------
    ///
    /// ************************* IMPORTANT *************************
    /// NKEY Seed printed below can be used sign and prove identity.
    /// NKEYs are sensitive and should be treated as secrets.
    ///
    /// -----BEGIN USER NKEY SEED-----
    /// SUAIO3FHUX5PNV2LQIIP7TZ3N4L7TX3W53MQGEIVYFIGA635OZCKEYHFLM
    /// ------END USER NKEY SEED------
    /// ";
    ///
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .credentials(creds)
    ///     .expect("failed to parse static creds")
    ///     .connect("connect.ngs.global")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn credentials(self, creds: &str) -> Result<Self> {
        Ok(self.inner.credentials(creds)?.into())
    }

    /// Loads root certificates by providing the path to them.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .add_root_certificates("mycerts.pem".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_root_certificates(self, path: PathBuf) -> ConnectOptions {
        self.inner.add_root_certificates(path).into()
    }

    /// Loads client certificate by providing the path to it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .add_client_certificate("cert.pem".into(), "key.pem".into())
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_client_certificate(self, cert: PathBuf, key: PathBuf) -> ConnectOptions {
        self.inner.add_client_certificate(cert, key).into()
    }

    /// Sets or disables TLS requirement. If TLS connection is impossible while
    /// `options.require_tls(true)` connection will return error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = si_data_nats::ConnectOptions::new()
    ///     .require_tls(true)
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn require_tls(self, is_required: bool) -> ConnectOptions {
        self.inner.require_tls(is_required).into()
    }

    /// Changes how tls connection is established.
    ///
    /// If `tls_first` is set, client will try to establish tls before getting info from the
    /// server. That requires the server to enable `handshake_first` option in the config.
    pub fn tls_first(self) -> ConnectOptions {
        self.inner.tls_first().into()
    }

    /// Sets how often Client sends PING message to the server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tokio::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .flush_interval(Duration::from_millis(100))
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ping_interval(self, ping_interval: Duration) -> ConnectOptions {
        self.inner.ping_interval(ping_interval).into()
    }

    /// Sets `no_echo` option which disables delivering messages that were published from the same
    /// connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .no_echo()
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_echo(self) -> ConnectOptions {
        self.inner.no_echo().into()
    }

    /// Sets the capacity for `Subscribers`. Exceeding it will trigger `slow consumer` error
    /// callback and drop messages. Default is set to 1024 messages buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .subscription_capacity(1024)
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscription_capacity(self, capacity: usize) -> ConnectOptions {
        self.inner.subscription_capacity(capacity).into()
    }

    /// Sets a timeout for the underlying TcpStream connection to avoid hangs and deadlocks.
    /// Default is set to 5 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .connection_timeout(tokio::time::Duration::from_secs(5))
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connection_timeout(self, timeout: Duration) -> ConnectOptions {
        self.inner.connection_timeout(timeout).into()
    }

    /// Sets a timeout for `Client::request`. Default value is set to 10 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .request_timeout(Some(std::time::Duration::from_secs(3)))
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn request_timeout(self, timeout: Option<Duration>) -> ConnectOptions {
        self.inner.request_timeout(timeout).into()
    }

    /// Registers an asynchronous callback for errors that are received over the wire from the
    /// server.
    ///
    /// # Examples
    /// As asynchronous callbacks are still not in `stable` channel, here are some examples how to
    /// work around this
    ///
    /// ## Basic
    /// If you don't need to move anything into the closure, simple signature can be used:
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .event_callback(|event| async move {
    ///         println!("event occurred: {}", event);
    ///     })
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Listening to specific event kind
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .event_callback(|event| async move {
    ///         match event {
    ///             si_data_nats::Event::Disconnected => println!("disconnected"),
    ///             si_data_nats::Event::Connected => println!("reconnected"),
    ///             si_data_nats::Event::ClientError(err) => println!("client error occurred: {}", err),
    ///             other => println!("other event happened: {}", other),
    ///         }
    ///     })
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Advanced
    /// If you need to move something into the closure, here's an example how to do that
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let (tx, mut _rx) = tokio::sync::mpsc::channel(1);
    /// si_data_nats::ConnectOptions::new()
    ///     .event_callback(move |event| {
    ///         let tx = tx.clone();
    ///         async move {
    ///             tx.send(event).await.unwrap();
    ///         }
    ///     })
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn event_callback<F, Fut>(self, cb: F) -> ConnectOptions
    where
        F: Fn(async_nats::Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + 'static + Send + Sync,
    {
        self.inner.event_callback(cb).into()
    }

    /// Registers a callback for a custom reconnect delay handler that can be used to define a
    /// backoff duration strategy.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .reconnect_delay_callback(|attempts| {
    ///         println!("no of attempts: {attempts}");
    ///         std::time::Duration::from_millis(std::cmp::min((attempts * 100) as u64, 8000))
    ///     })
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn reconnect_delay_callback<F>(self, cb: F) -> ConnectOptions
    where
        F: Fn(usize) -> Duration + Send + Sync + 'static,
    {
        self.inner.reconnect_delay_callback(cb).into()
    }

    /// By default, Client dispatches op's to the Client onto the channel with capacity of 128.
    /// This option enables overriding it.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// si_data_nats::ConnectOptions::new()
    ///     .client_capacity(256)
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn client_capacity(self, capacity: usize) -> ConnectOptions {
        self.inner.client_capacity(capacity).into()
    }

    /// Sets custom prefix instead of default `_INBOX`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .custom_inbox_prefix("CUSTOM")
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn custom_inbox_prefix<T: ToString>(self, prefix: T) -> ConnectOptions {
        self.inner.custom_inbox_prefix(prefix).into()
    }

    /// Sets the name for the client.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// si_data_nats::ConnectOptions::new()
    ///     .name("rust-service")
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn name<T: ToString>(self, name: T) -> ConnectOptions {
        self.inner.name(name.to_string()).into()
    }

    pub fn retry_on_initial_connect(self) -> ConnectOptions {
        self.inner.retry_on_initial_connect().into()
    }

    pub fn ignore_discovered_servers(self) -> ConnectOptions {
        self.inner.ignore_discovered_servers().into()
    }

    /// By default, client will pick random server to which it will try connect to. This option
    /// disables that feature, forcing it to always respect the order in which server addresses
    /// were passed.
    pub fn retain_servers_order(self) -> ConnectOptions {
        self.inner.retain_servers_order().into()
    }

    /// Allows passing custom rustls tls config.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let mut root_store = si_data_nats::rustls::RootCertStore::empty();
    ///
    /// root_store.add_parsable_certificates(
    ///     rustls_native_certs::load_native_certs()?
    ///         .into_iter()
    ///         .map(|cert| cert.0)
    ///         .collect::<Vec<Vec<u8>>>()
    ///         .as_ref(),
    /// );
    ///
    /// let tls_client = si_data_nats::rustls::ClientConfig::builder()
    ///     .with_safe_defaults()
    ///     .with_root_certificates(root_store)
    ///     .with_no_client_auth();
    ///
    /// let client = si_data_nats::ConnectOptions::new()
    ///     .require_tls(true)
    ///     .tls_client_config(tls_client)
    ///     .connect("tls://demo.nats.io")
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn tls_client_config(self, config: async_nats::rustls::ClientConfig) -> ConnectOptions {
        self.inner.tls_client_config(config).into()
    }

    /// Sets the initial capacity of the read buffer. Which is a buffer used to gather partial
    /// protocol messages.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// si_data_nats::ConnectOptions::new()
    ///     .read_buffer_capacity(65535)
    ///     .connect("demo.nats.io")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_buffer_capacity(self, size: u16) -> ConnectOptions {
        self.inner.read_buffer_capacity(size).into()
    }
}

impl From<async_nats::ConnectOptions> for ConnectOptions {
    fn from(inner: async_nats::ConnectOptions) -> Self {
        Self { inner }
    }
}
