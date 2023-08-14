use std::{path::PathBuf, time::Duration};

use super::{Client, Result};

/// Connect options.
#[derive(Debug, Default)]
pub struct Options {
    pub(crate) inner: async_nats::ConnectOptions,
}

impl Options {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Authenticate with NATS using a token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_token("t0k3n!")
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn with_token(token: String) -> Self {
        async_nats::ConnectOptions::with_token(token).into()
    }

    /// Authenticate with NATS using a username and password.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_user_and_password("derek", "s3cr3t!")
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn with_user_and_password(user: String, password: String) -> Self {
        async_nats::ConnectOptions::with_user_and_password(user, password).into()
    }

    /// Authenticate with NATS using a `.creds` file.
    ///
    /// This will open the provided file, load its creds, perform the desired authentication, and
    /// then zero the memory used to store the creds before continuing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_credentials_file("path/to/my.creds")
    ///     .connect("connect.ngs.global", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub async fn with_credentials_file(path: PathBuf) -> Result<Self> {
        Ok(async_nats::ConnectOptions::with_credentials_file(path)
            .await?
            .into())
    }

    /// Authenticate with NATS using a static credential str, using the creds file format.
    ///
    /// Note that this is more hazardous than using the above `with_credentials_file` method because it
    /// retains the secret in-memory for the lifetime of this client instead of zeroing the
    /// credentials after holding them for a very short time, as the `with_credentials_file` method
    /// does.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let creds =
    /// "-----BEGIN NATS USER JWT-----
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
    /// let nc = Options::with_credentials(creds)
    ///     .expect("failed to parse static creds")
    ///     .connect("connect.ngs.global", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn with_credentials(creds: &str) -> Result<Self> {
        Ok(async_nats::ConnectOptions::with_credentials(creds)?.into())
    }

    /// Set client certificate and private key files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .add_client_certificate("client-cert.pem", "client-key.pem")
    ///     .connect("nats://localhost:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn add_client_certificate(self, cert: PathBuf, key: PathBuf) -> Self {
        self.inner.add_client_certificate(cert, key).into()
    }

    /// Set the default TLS config that will be used for connections.
    ///
    /// Note that this is less secure than specifying TLS certificate file paths using the other
    /// methods on `Options`, which will avoid keeping raw key material in-memory and will zero
    /// memory buffers that temporarily contain key material during connection attempts.  This is
    /// intended to be used as a method of last-resort when providing well-known file paths is not
    /// feasible.
    ///
    /// To avoid version conflicts, the `rustls` version used by this crate is exported as
    /// `si_data_nats::rustls`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let mut tls_client_config = si_data_nats::rustls::ClientConfig::default();
    /// tls_client_config
    ///     .set_single_client_cert(
    ///         vec![si_data_nats::rustls::Certificate(b"MY_CERT".to_vec())],
    ///         si_data_nats::rustls::PrivateKey(b"MY_KEY".to_vec()),
    ///     );
    /// let nc = Options::new()
    ///     .tls_client_config(tls_client_config)
    ///     .connect("nats://localhost:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn tls_client_config(self, tls_client_config: async_nats::rustls::ClientConfig) -> Self {
        self.inner.tls_client_config(tls_client_config).into()
    }

    /// Add a name option to this configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .name("My App")
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn name(self, name: &str) -> Self {
        self.inner.name(name).into()
    }

    /// Select option to not deliver messages that we have published.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .no_echo()
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn no_echo(self) -> Self {
        self.inner.no_echo().into()
    }

    /// Establish a `Connection` with a NATS server.
    ///
    /// Multiple servers may be specified by separating them with commas.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    ///
    /// In the below case, the second server is configured to use TLS but the first one is not.
    /// Using the `tls_required` method can ensure that all servers are connected to with TLS, if
    /// that is your intention.
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .connect("nats://demo.nats.io:4222,tls://demo.nats.io:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub async fn connect(
        self,
        nats_url: impl Into<String>,
        subject_prefix: Option<String>,
    ) -> Result<Client> {
        Client::connect_with_options(nats_url, subject_prefix, self).await
    }

    /// Set a callback to be executed for calculating the backoff duration to wait before a server
    /// reconnection attempt.
    ///
    /// The function takes the number of reconnects as an argument and returns the `Duration` that
    /// should be waited before making the next connection attempt.
    ///
    /// It is recommended that some random jitter is added to your returned `Duration`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # use std::time::Duration;
    /// let nc = Options::new()
    ///     .reconnect_delay_callback(|c| {
    ///         Duration::from_millis(std::cmp::min((c * 100) as u64, 8000))
    ///     })
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn reconnect_delay_callback<F>(self, cb: F) -> Self
    where
        F: Fn(usize) -> Duration + Send + Sync + 'static,
    {
        self.inner.reconnect_delay_callback(cb).into()
    }

    /// Setting this requires that TLS be set for all server connections.
    ///
    /// If you only want to use TLS for some server connections, you may declare them separately in
    /// the connect string by prefixing them with `tls://host:port` instead of `nats://host:port`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .require_tls(true)
    ///     .connect("tls://demo.nats.io:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn require_tls(self, tls_required: bool) -> Self {
        self.inner.require_tls(tls_required).into()
    }

    /// Adds a root certificate file.
    ///
    /// The file must be PEM encoded. All certificates in the file will be used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .add_root_certificates("my-certs.pem")
    ///     .connect("tls://demo.nats.io:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn add_root_certificates(self, path: PathBuf) -> Self {
        self.inner.add_root_certificates(path).into()
    }
}

impl From<async_nats::ConnectOptions> for Options {
    fn from(inner: async_nats::ConnectOptions) -> Self {
        Self { inner }
    }
}
