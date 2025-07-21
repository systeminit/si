//! This crate provides a client for interacting with AWS ACM PCA

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    fmt::Debug,
    time::Duration,
};

use aws_sdk_acmpca::{
    error::{
        BuildError,
        SdkError,
    },
    operation::get_certificate::GetCertificateError,
    primitives::Blob,
    types::{
        SigningAlgorithm,
        Validity,
        ValidityPeriodType,
    },
};
use rcgen::{
    CertificateParams,
    DistinguishedName,
    DnType,
    ExtendedKeyUsagePurpose,
    KeyPair,
    KeyUsagePurpose,
};
use si_aws_config::{
    AwsConfig,
    AwsConfigError,
};
use si_tls::{
    CertificateSource,
    KeySource,
};
use telemetry::prelude::*;
use thiserror::Error;

const DEFAULT_CERT_VALIDITY: i64 = 7;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PrivateCertManagerClientError {
    #[error("AWS Config Error error: {0}")]
    AwsConfig(#[from] AwsConfigError),
    #[error("AWS ACM PCA error: {0}")]
    AwsPrivateCertManager(String),
    #[error("Certificate Authority not found with arn: {0}")]
    CertificateAuthorityNotFound(String),
    #[error("Failed to get cert: {0} from ca: {1}")]
    GetCertificate(String, String),
    #[error("Failed to issue cert from ca: {0}")]
    IssueCertificate(String),
    #[error("RCGen error: {0}")]
    RcGen(#[from] rcgen::Error),
    #[error("Validity build error: {0}")]
    Validity(#[from] BuildError),
}

impl PrivateCertManagerClientError {
    fn from_sdk_error<T: Debug>(error: SdkError<T>) -> Self {
        PrivateCertManagerClientError::AwsPrivateCertManager(format!("{error:?}"))
    }
}

type PrivateCertManagerClientResult<T> = Result<T, PrivateCertManagerClientError>;

/// A client for communicating with ssm.
#[derive(Debug, Clone)]
pub struct PrivateCertManagerClient {
    inner: Box<aws_sdk_acmpca::Client>,
}

impl PrivateCertManagerClient {
    /// Creates a new [client for interacting with ACM PCA](PrivateCertManagerClient).
    #[instrument(name = "private_cert_manager_client.new", level = "info")]
    pub async fn new() -> PrivateCertManagerClientResult<Self> {
        let config = AwsConfig::from_env().await?;
        let client = aws_sdk_acmpca::Client::new(&config);
        Ok(Self {
            inner: Box::new(client),
        })
    }

    /// Gets a CA
    pub async fn get_certificate_authority(
        &self,
        ca_arn: String,
    ) -> PrivateCertManagerClientResult<CertificateSource> {
        let result = self
            .inner
            .get_certificate_authority_certificate()
            .certificate_authority_arn(ca_arn.clone())
            .send()
            .await
            .map_err(PrivateCertManagerClientError::from_sdk_error)?;

        let cert = result.certificate().ok_or(
            PrivateCertManagerClientError::CertificateAuthorityNotFound(ca_arn),
        )?;

        Ok(CertificateSource::AsString(cert.to_string()))
    }

    /// Issues and retrieves a new cert from the CA
    pub async fn get_new_cert_from_ca(
        &self,
        ca_arn: String,
        for_app: String,
        for_service: String,
    ) -> PrivateCertManagerClientResult<(CertificateSource, KeySource)> {
        let (csr, key) = generate_instance_cert(for_app, for_service)?;
        let result = self
            .inner
            .issue_certificate()
            .certificate_authority_arn(ca_arn.clone())
            .signing_algorithm(SigningAlgorithm::Sha256Withrsa)
            .csr(Blob::new(csr.as_bytes()))
            .validity(
                Validity::builder()
                    .r#type(ValidityPeriodType::Days)
                    .value(DEFAULT_CERT_VALIDITY)
                    .build()?,
            )
            .send()
            .await
            .map_err(PrivateCertManagerClientError::from_sdk_error)?;

        let cert_arn =
            result
                .certificate_arn()
                .ok_or(PrivateCertManagerClientError::IssueCertificate(
                    ca_arn.clone(),
                ))?;

        let max_attempts = 5;
        for attempt in 0..max_attempts {
            match self
                .inner
                .get_certificate()
                .certificate_authority_arn(ca_arn.clone())
                .certificate_arn(cert_arn)
                .send()
                .await
            {
                Ok(result) => {
                    if let Some(cert) = result.certificate() {
                        return Ok((
                            CertificateSource::AsString(cert.to_string()),
                            KeySource::AsString(key),
                        ));
                    }
                }
                Err(err) => {
                    if let Some(GetCertificateError::RequestInProgressException(_)) =
                        err.as_service_error()
                    {
                        if attempt < max_attempts - 1 {
                            let delay = Duration::from_secs(2u64.pow(attempt as u32));
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Err(PrivateCertManagerClientError::from_sdk_error(err));
                }
            }
        }

        Err(PrivateCertManagerClientError::GetCertificate(
            ca_arn,
            cert_arn.to_string(),
        ))
    }
}

fn generate_instance_cert(
    app_name: String,
    ou_name: String,
) -> PrivateCertManagerClientResult<(String, String)> {
    let key_pair = KeyPair::generate()?;

    let mut params =
        CertificateParams::new(vec![format!("{}-{}.systeminit.com", ou_name, app_name)])?;

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, app_name);
    dn.push(DnType::OrganizationName, "System Initiative");
    dn.push(DnType::OrganizationalUnitName, ou_name);

    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ClientAuth);

    params.distinguished_name = dn;

    Ok((
        params.serialize_request(&key_pair)?.pem()?,
        key_pair.serialize_pem(),
    ))
}
