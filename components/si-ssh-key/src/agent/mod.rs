use crate::model::entity::EntityEvent;
use si_cea::agent::dispatch::Dispatch;
use si_cea::{
    gen_dispatch, gen_dispatch_keys, gen_dispatch_setup, gen_dispatcher, CeaResult,
    MqttAsyncClientInternal,
};
use si_data::Db;

gen_dispatcher!(self_ident: self);

#[async_trait::async_trait]
impl Dispatch<EntityEvent> for Dispatcher {
    gen_dispatch_keys!(self);

    async fn setup(&mut self, db: &Db) -> CeaResult<()> {
        gen_dispatch_setup!(
            self,
            db,
            {
                integration_name: "global",
                integration_service_name: "ssh_key",
                dispatch[
                    ("create", global::create),
                    ("sync", global::sync)
                ]
            },
            {
                integration_name: "aws",
                integration_service_name: "ec2",
                dispatch[
                    ("create", aws::create),
                    ("sync", aws::sync)
                ]
            }
        );
        Ok(())
    }

    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
        integration_service_id: String,
        action_name: String,
    ) -> CeaResult<()> {
        gen_dispatch!(
            self,
            mqtt_client,
            entity_event,
            integration_service_id,
            action_name,
            dispatch[
                global::create,
                global::sync,
                aws::create,
                aws::sync
            ]
        );
        Ok(())
    }
}

pub mod global {
    use crate::model::entity::{EntityEvent, KeyFormat, KeyType};
    use si_cea::{
        spawn_command, CaptureOutput, CeaError, CeaResult, EntityEvent as _,
        MqttAsyncClientInternal,
    };
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::process::Command;
    use tracing::{debug, debug_span};
    use tracing_futures::Instrument as _;

    pub async fn sync(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            debug!(?entity_event);
            entity_event.output_entity = entity_event.input_entity.clone();
            entity_event.log("Synchronized State");
            Ok(())
        }
        .instrument(debug_span!("ssh_key_global_sync"))
        .await
    }

    pub async fn create(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            debug!(?entity_event);
            let tempdir = TempDir::new()?;
            let filename = tempdir.path().join("newkey");

            // Before this gets used in anger, the user needs to give us the passphrase
            let input_entity = entity_event
                .input_entity
                .as_ref()
                .ok_or(CeaError::MissingInputEntity)?;
            let key_type = KeyType::from_i32(input_entity.key_type)
                .ok_or(CeaError::ValidationError("key type is invalid".to_string()))?;

            let key_format = KeyFormat::from_i32(input_entity.key_format).ok_or(
                CeaError::ValidationError("key format is invalid".to_string()),
            )?;
            let mut ssh_keygen_cmd = Command::new("ssh-keygen");
            ssh_keygen_cmd.current_dir(tempdir.path());
            ssh_keygen_cmd.arg("-t");
            ssh_keygen_cmd.arg(format!("{}", key_type));
            ssh_keygen_cmd.arg("-m");
            ssh_keygen_cmd.arg(format!("{}", key_format));
            ssh_keygen_cmd.arg("-b");
            ssh_keygen_cmd.arg(format!("{}", input_entity.bits));
            ssh_keygen_cmd.arg("-C");
            ssh_keygen_cmd.arg(&input_entity.name[..]);
            ssh_keygen_cmd.arg("-f");
            ssh_keygen_cmd.arg(filename.to_string_lossy().as_ref());
            ssh_keygen_cmd.arg("-N");
            ssh_keygen_cmd.arg("");

            spawn_command(
                &mqtt_client,
                ssh_keygen_cmd,
                entity_event,
                CaptureOutput::None,
            )
            .await?
            .success()?;

            entity_event.output_entity = entity_event.input_entity.clone();
            {
                let output_entity = entity_event
                    .output_entity
                    .as_mut()
                    .ok_or(CeaError::MissingInputEntity)?;

                let private_key = fs::read(&filename).await?;
                output_entity.private_key = String::from_utf8(private_key)?;

                let public_key_name = format!("{}.pub", filename.display());
                let public_key = fs::read(&public_key_name).await?;
                output_entity.public_key = String::from_utf8(public_key)?;
            }

            let mut ssh_fingerprint_cmd = Command::new("ssh-keygen");
            ssh_fingerprint_cmd.current_dir(tempdir.path());
            ssh_fingerprint_cmd.arg("-l");
            ssh_fingerprint_cmd.arg("-f");
            ssh_fingerprint_cmd.arg(filename.to_string_lossy().as_ref());
            let mut ssh_fingerprint_out = spawn_command(
                &mqtt_client,
                ssh_fingerprint_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

            let mut ssh_babble_cmd = Command::new("ssh-keygen");
            ssh_babble_cmd.current_dir(tempdir.path());
            ssh_babble_cmd.arg("-B");
            ssh_babble_cmd.arg("-f");
            ssh_babble_cmd.arg(filename.to_string_lossy().as_ref());
            let mut ssh_babble_out = spawn_command(
                &mqtt_client,
                ssh_babble_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

            let mut ssh_random_cmd = Command::new("ssh-keygen");
            ssh_random_cmd.current_dir(tempdir.path());
            ssh_random_cmd.arg("-l");
            ssh_random_cmd.arg("-v");
            ssh_random_cmd.arg("-f");
            ssh_random_cmd.arg(filename.to_string_lossy().as_ref());
            let mut ssh_random_out = spawn_command(
                &mqtt_client,
                ssh_random_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;
            {
                let output_entity = entity_event
                    .output_entity
                    .as_mut()
                    .ok_or(CeaError::MissingOutputEntity)?;
                output_entity.fingerprint = ssh_fingerprint_out.try_stdout()?;
                output_entity.bubble_babble = ssh_babble_out.try_stdout()?;
                output_entity.random_art = ssh_random_out.try_stdout()?;
            }
            entity_event.succeeded();
            entity_event.send_via_mqtt(&mqtt_client).await?;
            Ok(())
        }
        .instrument(debug_span!("ssh_key_global_create"))
        .await
    }
}

pub mod aws {
    use crate::model::entity::{EntityEvent, KeyFormat};
    use si_cea::{
        spawn_command, CaptureOutput, CeaError, CeaResult, EntityEvent as _,
        MqttAsyncClientInternal,
    };
    use si_external_api_gateway::aws::ec2;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;
    use tokio::process::Command;
    use tracing::{debug, debug_span};
    use tracing_futures::Instrument as _;

    pub async fn create(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            debug!(?entity_event);
            // More evidence this should be refactored - why connect multiple times, rather than
            // multiplexing? Even if we need N connections, better to manage it higher up.
            let mut ec2 = ec2::Ec2Client::connect("http://localhost:4001").await?;

            entity_event.output_entity = entity_event.input_entity.clone();

            let key_name = if entity_event.input_entity().is_some() {
                entity_event.input_entity().as_ref().unwrap().id.to_string()
            } else {
                return Err(CeaError::MissingInputEntity);
            };
            entity_event.log("Creating Key Pair in EC2");
            let result = ec2
                .create_key_pair(ec2::CreateKeyPairRequest {
                    context: Some(ec2::Context {
                        billing_account_id: entity_event.billing_account_id.to_string(),
                        organization_id: entity_event.organization_id.to_string(),
                        workspace_id: entity_event.workspace_id.to_string(),
                        ..Default::default()
                    }),
                    key_name,
                    dry_run: false,
                })
                .await?
                .into_inner();
            debug!(?result, "ec2_create_keypair");
            if result.error.is_some() {
                let e = result.error.as_ref().unwrap();
                entity_event.error_log("Request failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                return Err(CeaError::ExternalRequest);
            }
            entity_event.log("Creation successful!\n");
            entity_event.log(format!("Request ID: {}\n", result.request_id));
            entity_event.log(format!("Key Fingerprint: {}\n", result.key_fingerprint));
            entity_event.log("Key Material:\n");
            entity_event.log(format!("{}\n", result.key_material));
            entity_event.log(format!("Key Name: {}\n", result.key_name));
            entity_event.log(format!("Key Pair ID: {}\n", result.key_pair_id));

            let tempdir = TempDir::new()?;
            let filename = tempdir.path().join("newkey");
            tokio::fs::write(&filename, &result.key_material).await?;
            let file_metadata = tokio::fs::metadata(&filename).await?;
            let mut file_perms = file_metadata.permissions();
            file_perms.set_mode(0o600);
            tokio::fs::set_permissions(&filename, file_perms).await?;

            let key_format =
                match KeyFormat::from_i32(entity_event.input_entity().as_ref().unwrap().key_format)
                {
                    Some(key_format) => key_format,
                    None => {
                        return Err(CeaError::ValidationError(
                            "incorrect key format".to_string(),
                        ))
                    }
                };

            let mut ssh_public_key_cmd = Command::new("ssh-keygen");
            ssh_public_key_cmd.current_dir(tempdir.path());
            ssh_public_key_cmd.arg("-e");
            ssh_public_key_cmd.arg("-f");
            ssh_public_key_cmd.arg(filename.to_string_lossy().as_ref());
            ssh_public_key_cmd.arg("-m");
            ssh_public_key_cmd.arg(format!("{}", key_format));
            let mut ssh_public_key_out = spawn_command(
                &mqtt_client,
                ssh_public_key_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

            let filename_pub = tempdir.path().join("newkey.pub");
            tokio::fs::write(
                &filename_pub,
                &ssh_public_key_out.stdout().unwrap_or(&"".to_string()),
            )
            .await?;
            let pub_file_metadata = tokio::fs::metadata(&filename_pub).await?;
            let mut pub_file_perms = pub_file_metadata.permissions();
            pub_file_perms.set_mode(0o644);
            tokio::fs::set_permissions(&filename_pub, pub_file_perms).await?;

            let mut ssh_fingerprint_cmd = Command::new("ssh-keygen");
            ssh_fingerprint_cmd.current_dir(tempdir.path());
            ssh_fingerprint_cmd.arg("-l");
            ssh_fingerprint_cmd.arg("-f");
            ssh_fingerprint_cmd.arg(filename.to_string_lossy().as_ref());
            let mut ssh_fingerprint_out = spawn_command(
                &mqtt_client,
                ssh_fingerprint_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

            let mut ssh_babble_cmd = Command::new("ssh-keygen");
            ssh_babble_cmd.current_dir(tempdir.path());
            ssh_babble_cmd.arg("-B");
            ssh_babble_cmd.arg("-f");
            ssh_babble_cmd.arg(filename.to_string_lossy().as_ref());
            let mut ssh_babble_out = spawn_command(
                &mqtt_client,
                ssh_babble_cmd,
                entity_event,
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

            let output_entity = entity_event
                .output_entity
                .as_mut()
                .ok_or(CeaError::MissingOutputEntity)?;
            output_entity.private_key = result.key_material.to_string();
            output_entity.public_key = ssh_public_key_out.try_stdout()?;
            output_entity.fingerprint = ssh_fingerprint_out.try_stdout()?;
            output_entity.bubble_babble = ssh_babble_out.try_stdout()?;
            entity_event.succeeded();
            entity_event.send_via_mqtt(&mqtt_client).await?;

            Ok(())
        }
        .instrument(debug_span!("ssh_key_aws_create"))
        .await
    }

    pub async fn sync(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            let mut ec2 = ec2::Ec2Client::connect("http://localhost:4001").await?;
            entity_event.log("Synchronizing Key Pair in EC2");
            let result = ec2
                .describe_key_pairs(ec2::DescribeKeyPairsRequest {
                    context: Some(ec2::Context {
                        billing_account_id: entity_event.billing_account_id.to_string(),
                        organization_id: entity_event.organization_id.to_string(),
                        workspace_id: entity_event.workspace_id.to_string(),
                        ..Default::default()
                    }),
                    key_names: vec![entity_event.entity_id.clone()],
                })
                .await?
                .into_inner();
            if result.error.is_some() {
                let e = result.error.as_ref().unwrap();
                entity_event.error_log("Request failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                return Err(CeaError::ExternalRequest);
            }
            if result.key_pairs.len() > 0 {
                entity_event.log("Sync successful!\n");
                entity_event.log(format!(
                    "Fingerprint: {}\n",
                    result.key_pairs[0].key_fingerprint
                ));
                entity_event.output_entity = entity_event.input_entity.clone();
                entity_event.log("Synchronized State");
            }
            Ok(())
        }
        .instrument(debug_span!("ssh_key_aws_sync"))
        .await
    }
}
