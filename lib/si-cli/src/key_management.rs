use crate::{CliResult, SiCliError};
use base64::{engine::general_purpose, Engine};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use si_crypto::SymmetricCryptoService;
use sodiumoxide::crypto::box_;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Credentials {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_session_token: Option<String>,
    pub aws_endpoint_url: Option<String>,
    pub docker_hub_user_name: Option<String>,
    pub docker_hub_credential: Option<String>,
    pub si_email: Option<String>,
}

const JWT_SIGNING_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAuoogz71y+EO3tmEAiHCD
90A/AnF6idrD31VY8bkpwYS51UGIlkdUna/yQo6XiXj10rhNkVEAzjwmfoGJbj59
gyFcFZtzsDuI6lOVn6XvSjYuHFPqeyYn8QFlIYBJuMpgiRPwuQK+JN8Ke5PdrOL3
fwL93i9BGJzczuDVK5McAyeuJsuQm8DfJk/BaQYTAjBW2ZI/3IxEUpsOrW9abpFe
Uejnd0fWepbo1M84US4lWAJKXS6+VeUnNaS/msl0T980DnO5oocdY6gWmQPNWikB
JkUxQGNUIfwVMelUgvML8ZUk0LFIphIlJzzcA0nzN6Y1otEezxpfNYVRK+rF3m27
TLmHvISaqUJ58AqBC22YOdgDOY2hkdS9tOQfcr+GJg+REKlfo4nUuodaB0imdwas
nLzqa4veBKPeE37Zw1x2Uph4dtbfEyNHGORTLUqP/j2KP8CLY82Kj40vSVk0l3RN
ERf4e/qM52I2EDYzHej7uYsheouTrRUf9IAyBEkU7WO4L2Y6N/pgyiF/5I5aHcY2
l8l9ozan8igwKyNY7EiB4T8fA3yEkpNhwQAVlF897dUTanb2SjyFBpZxx+POZDpC
fJwjkI25wNiOHD7LI8nWUqXOM0ZcQQ/4HJwG9IT6flvRQwLi9UrC8FTos4jPeZcA
T7Pftf1OUGsDQsmx/eAS4GUCAwEAAQ==
-----END PUBLIC KEY-----";

const DEV_POSTGRES_ROOT_CRT: &str = "-----BEGIN CERTIFICATE-----
MIIDSzCCAjOgAwIBAgIIAhGEx24hny0wDQYJKoZIhvcNAQELBQAwIDEeMBwGA1UE
AxMVbWluaWNhIHJvb3QgY2EgMDIxMTg0MCAXDTI0MDEwMjE4MjEyOVoYDzIxMjQw
MTAyMTgyMTI5WjAgMR4wHAYDVQQDExVtaW5pY2Egcm9vdCBjYSAwMjExODQwggEi
MA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDoByK7UWZsjrKw31m6SwwiIplQ
d9MvfcD0+Gvvx1Lx0DNNangU/xqc0K5puNAPqwOToKLihbnJwqjXNUazXaM3ojcn
dkeRs3Cz7KVZkiu+sXqiecMXL2nFkNBSPhiXRumFOw3Qhz6v6dCMZxXcHUIgeBnp
q0oCQprCSt+S2TpgsPrNxCbByB/ICfI/rT+/QOVeJjqcJe5RMFAIR8pxaBcTlScL
fwj7PPd2eW/jZ1+pVfYYqKwbLwv14faElMZ2MZwRANDHo7lPA4riB6m7yfRXN7Cn
bwKGMJFPVoO71UL/ArAr/8bTEf1FZejMPzG7oEqXSGyYTLi0ybLr7G6QVqpJAgMB
AAGjgYYwgYMwDgYDVR0PAQH/BAQDAgKEMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggr
BgEFBQcDAjASBgNVHRMBAf8ECDAGAQH/AgEAMB0GA1UdDgQWBBQHGAB8jOdnvpxu
lS8NlI2L0jRxUzAfBgNVHSMEGDAWgBQHGAB8jOdnvpxulS8NlI2L0jRxUzANBgkq
hkiG9w0BAQsFAAOCAQEANH3AydMZZGDeTP7SaHPc/+XsRu/0cTTCXd0Jr56RbnPg
Y2B5jq2WfN7DO7uCva2N4Q6oILMH8JBonFgGh5/LKrFrfzOQBuHWTqyA55QfkYlY
bZv3zWj3X3UqQcKWJW21P/cWiLiABQD3ahTNaWop1yu2nWvjTosE6/iXloOfTHfx
XEyztSRLxYGoqrba7MW2RKAo31ENrZVKtI+X8vjxJa1s/sqLQUbyCDVF+l6tsR0T
kN1/Vtm5EUdiEvDRETI4pQ9b1FSXulSaQJVNFWONXjrb5fhXpnPQ+zG3COdKbKz8
OGyod3OoVim6KL7RFkjqu2SYWrc5/tmOn+VWBEim/w==
-----END CERTIFICATE-----";

pub async fn ensure_encryption_keys() -> CliResult<()> {
    let (public_key, secret_key) = box_::gen_keypair();

    let si_data_dir = get_si_data_dir().await?;
    let secret_key_path = si_data_dir.join("cyclone_encryption.key");
    if !secret_key_path.exists() {
        let mut file = File::create(&secret_key_path)?;
        file.write_all(&secret_key.0)?;
    }

    let public_key_path = si_data_dir.join("decryption.key");
    if !public_key_path.exists() {
        let mut file = File::create(&public_key_path)?;
        file.write_all(&public_key.0)?;
    }
    Ok(())
}

pub async fn ensure_symmetric_crypto_key() -> CliResult<()> {
    let si_data_dir = get_si_data_dir().await?;
    let active_key_path = si_data_dir.join("donkey.key");
    if !active_key_path.exists() {
        SymmetricCryptoService::generate_key()
            .save(active_key_path)
            .await?;
    }

    Ok(())
}

pub async fn ensure_jwt_public_signing_key() -> CliResult<()> {
    let si_data_dir = get_si_data_dir().await?;
    let jwt_public_signing_key = si_data_dir.join("jwt_signing_public_key.pem");
    if !jwt_public_signing_key.exists() {
        let mut file = File::create(&jwt_public_signing_key)?;
        file.write_all(JWT_SIGNING_KEY.as_bytes())?;
    }

    Ok(())
}

pub async fn ensure_postgres_root_cert() -> CliResult<()> {
    let si_data_dir = get_si_data_dir().await?;
    let postgres_root_cert = si_data_dir.join("dev.postgres.root.crt");
    if !postgres_root_cert.exists() {
        let mut file = File::create(&postgres_root_cert)?;
        file.write_all(DEV_POSTGRES_ROOT_CRT.as_bytes())?;
    }

    Ok(())
}

pub async fn write_veritech_credentials(
    credentials: &Credentials,
    credentials_path: PathBuf,
) -> CliResult<()> {
    let creds = toml::to_string(credentials).expect("Error creating Toml Structure");
    fs::write(credentials_path, creds).expect("Error writing to credentials file");
    Ok(())
}

pub async fn get_credentials() -> CliResult<Credentials> {
    let credentials_file_path = get_si_data_dir().await?.join("si_credentials.toml");
    match fs::read_to_string(credentials_file_path) {
        Ok(found_contents) => Ok(toml::from_str(found_contents.as_str())?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Credentials::default()),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_user_email() -> CliResult<String> {
    let data_dir_exists = get_si_data_dir().await;
    if data_dir_exists.is_err() {
        // If the data_dir doesn't exist then we should default to sally for now
        return Ok("sally@systeminit.com".to_string());
    }

    let credentials = get_credentials().await?;
    if let Some(email) = credentials.si_email {
        Ok(email)
    } else {
        Ok("sally@systeminit.com".to_string())
    }
}

pub async fn format_credentials_for_veritech() -> CliResult<Vec<String>> {
    let raw_creds = get_credentials().await?;
    let mut creds = Vec::new();

    if let Some(url) = raw_creds.aws_endpoint_url {
        creds.push(format!("AWS_ENDPOINT_URL={}", url));
    }

    if raw_creds.docker_hub_user_name.is_some() && raw_creds.docker_hub_credential.is_some() {
        let mut username = "".to_string();
        let mut credential = "".to_string();
        if let Some(user_name) = raw_creds.docker_hub_user_name {
            username = user_name
        }
        if let Some(cred) = raw_creds.docker_hub_credential {
            credential = cred
        }
        let docker_creds = format!("{}:{}", username, credential);
        let mut buf = String::new();
        general_purpose::STANDARD.encode_string(docker_creds.as_bytes(), &mut buf);

        creds.push(format!("DOCKER_AUTHENTICATION={}", buf));
    }

    Ok(creds)
}

pub async fn does_credentials_file_exist() -> CliResult<bool> {
    Ok(get_si_data_dir()
        .await?
        .join("si_credentials.toml")
        .exists())
}

pub async fn get_si_data_dir() -> Result<PathBuf, SiCliError> {
    if let Some(base_dirs) = BaseDirs::new() {
        let si_data_dir = base_dirs.data_dir().join("SI");
        let si_dir_exists = si_data_dir.as_path().is_dir();
        if !si_dir_exists {
            fs::create_dir_all(si_data_dir.as_path())?;
        }
        return Ok(si_data_dir);
    }

    Err(SiCliError::MissingDataDir())
}
