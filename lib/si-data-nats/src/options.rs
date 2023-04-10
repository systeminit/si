use std::{io, path::Path, time::Duration};

use super::{Client, Result};

/// Connect options.
#[derive(Debug, Default)]
pub struct Options {
    pub(crate) inner: nats_client::Options,
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
    pub fn with_token(token: &str) -> Self {
        nats_client::Options::with_token(token).into()
    }

    /// Authenticate with NATS using a username and password.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_user_pass("derek", "s3cr3t!")
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn with_user_pass(user: &str, password: &str) -> Self {
        nats_client::Options::with_user_pass(user, password).into()
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
    /// let nc = Options::with_credentials("path/to/my.creds")
    ///     .connect("connect.ngs.global", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn with_credentials(path: impl AsRef<Path>) -> Self {
        nats_client::Options::with_credentials(path).into()
    }

    /// Authenticate with NATS using a static credential str, using the creds file format.
    ///
    /// Note that this is more hazardous than using the above `with_credentials` method because it
    /// retains the secret in-memory for the lifetime of this client instead of zeroing the
    /// credentials after holding them for a very short time, as the `with_credentials` method
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
    /// let nc = Options::with_static_credentials(creds)
    ///     .expect("failed to parse static creds")
    ///     .connect("connect.ngs.global", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn with_static_credentials(creds: &str) -> Result<Self> {
        Ok(nats_client::Options::with_static_credentials(creds)?.into())
    }

    /// Authenticate with a function that loads user JWT and a signature function.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let kp = nkeys::KeyPair::from_seed(seed).unwrap();
    ///
    /// fn load_jwt() -> std::io::Result<String> {
    ///     todo!()
    /// }
    ///
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_jwt(load_jwt, move |nonce| kp.sign(nonce).unwrap())
    ///     .connect("localhost", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn with_jwt<J, S>(jwt_cb: J, sig_cb: S) -> Self
    where
        J: Fn() -> io::Result<String> + Send + Sync + 'static,
        S: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static,
    {
        nats_client::Options::with_jwt(jwt_cb, sig_cb).into()
    }

    /// Authenticate with NATS using a public key and a signature function.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let nkey = "UAMMBNV2EYR65NYZZ7IAK5SIR5ODNTTERJOBOF4KJLMWI45YOXOSWULM";
    /// let seed = "SUANQDPB2RUOE4ETUA26CNX7FUKE5ZZKFCQIIW63OX225F2CO7UEXTM7ZY";
    /// let kp = nkeys::KeyPair::from_seed(seed).unwrap();
    ///
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::with_nkey(nkey, move |nonce| kp.sign(nonce).unwrap())
    ///     .connect("localhost", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn with_nkey<F>(nkey: &str, sig_cb: F) -> Self
    where
        F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static,
    {
        nats_client::Options::with_nkey(nkey, sig_cb).into()
    }

    /// Set client certificate and private key files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .client_cert("client-cert.pem", "client-key.pem")
    ///     .connect("nats://localhost:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn client_cert(self, cert: impl AsRef<Path>, key: impl AsRef<Path>) -> Self {
        self.inner.client_cert(cert, key).into()
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
    pub fn tls_client_config(self, tls_client_config: nats_client::rustls::ClientConfig) -> Self {
        self.inner.tls_client_config(tls_client_config).into()
    }

    /// Add a name option to this configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .with_name("My App")
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn with_name(self, name: &str) -> Self {
        self.inner.with_name(name).into()
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

    /// Set the maximum number of reconnect attempts.  If no servers remain that are under this
    /// threshold, then no further reconnect shall be attempted.  The reconnect attempt for a
    /// server is reset upon successful connection.
    ///
    /// If None then there is no maximum number of attempts.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .max_reconnects(3)
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn max_reconnects<T: Into<Option<usize>>>(self, max_reconnects: T) -> Self {
        self.inner.max_reconnects(max_reconnects).into()
    }

    /// Set the maximum amount of bytes to buffer when accepting outgoing traffic in disconnected
    /// mode.
    ///
    /// The default value is 8mb.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .reconnect_buffer_size(64 * 1024)
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn reconnect_buffer_size(self, reconnect_buffer_size: usize) -> Self {
        self.inner
            .reconnect_buffer_size(reconnect_buffer_size)
            .into()
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

    /// Set a callback to be executed when connectivity to a server has been lost.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .disconnect_callback(|| println!("connection has been lost"))
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn disconnect_callback<F>(self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.disconnect_callback(cb).into()
    }

    /// Set a callback to be executed when connectivity to a server has been reestablished.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .reconnect_callback(|| println!("connection has been reestablished"))
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn reconnect_callback<F>(self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.reconnect_callback(cb).into()
    }

    // This is no longer valid - the upstream splits config between Nats and JetStream
    // -- Adam
    //
    // we aren't using JetStream, so...
    // Set a custom `JetStream` API prefix. This is useful when using `JetStream` through
    // exports/imports.
    //
    // # Examples
    //
    // ```no_run
    // # use si_data_nats::Options; tokio_test::block_on(async {
    // let nc = Options::new()
    //     .jetstream_api_prefix("some_exported_prefix".to_string())
    //     .connect("demo.nats.io", None)
    //     .await?;
    // nc.drain().await?;
    // # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    // ```
    //#[must_use]
    //pub fn jetstream_api_prefix(self, jetstream_prefix: String) -> Self {
    //    self.inner.jetstream_api_prefix(jetstream_prefix).into()
    //}

    /// Set a callback to be executed when the client has been closed due to exhausting reconnect
    /// retries to known servers or by completing a drain request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// let nc = Options::new()
    ///     .close_callback(|| println!("connection has been closed"))
    ///     .connect("demo.nats.io", None)
    ///     .await?;
    /// nc.drain().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn close_callback<F>(self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.close_callback(cb).into()
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
    ///     .tls_required(true)
    ///     .connect("tls://demo.nats.io:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn tls_required(self, tls_required: bool) -> Self {
        self.inner.tls_required(tls_required).into()
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
    ///     .add_root_certificate("my-certs.pem")
    ///     .connect("tls://demo.nats.io:4443", None)
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn add_root_certificate(self, path: impl AsRef<Path>) -> Self {
        self.inner.add_root_certificate(path).into()
    }
}

impl From<nats_client::Options> for Options {
    fn from(inner: nats_client::Options) -> Self {
        Self { inner }
    }
}
