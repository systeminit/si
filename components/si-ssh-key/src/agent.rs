use tempfile::TempDir;
use tokio::fs;
use tokio::net::process::Command;
use tracing::{event, span, Level};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::ssh_key::{Component, Entity, KeyFormat, KeyType};

// This is a simple agent design; it doesn't have any different components based on various
// integrations. In a complex example, this struct might do much more complex things - dispatch
// on to a network, a pub-sub bus, etc.
pub struct Agent {
    component: Component,
    entity: Entity,
}

impl Agent {
    pub fn new(component: Component, entity: Entity) -> Agent {
        Agent { component, entity }
    }

    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn into_entity(self) -> Entity {
        self.entity
    }

    pub async fn create(&mut self) -> Result<()> {
        let span = span!(Level::INFO, "agent_create");
        let _start_span = span.enter();

        // Fill in the default fields
        self.entity.id = format!("entity:sshkey:{}", Uuid::new_v4());
        self.entity.type_name = "entity:ssh_key".to_string();
        self.entity.display_type_name = "SSH Key".to_string();
        self.entity.component_id = self.component.id.clone();
        self.entity.display_name = self.entity.name.clone();
        self.entity.key_type = self.component.key_type;
        self.entity.key_format = self.component.key_format;
        self.entity.bits = self.component.bits;
        self.entity.comment = self.component.name.clone();
        self.entity.natural_key = format!(
            "{}/{}/{}",
            self.component.natural_key, self.entity.type_name, self.entity.name
        );

        let tempdir = TempDir::new()?;
        let filename = tempdir.path().join("newkey");

        event!(Level::INFO, ?self.component, ?self.entity, "Creating new ssh-key");

        let key_type = format!(
            "{}",
            KeyType::from_i32(self.component.key_type).ok_or(Error::InvalidKeyType)?
        );
        let key_format = format!(
            "{}",
            KeyFormat::from_i32(self.component.key_format).ok_or(Error::InvalidKeyFormat)?
        );

        // Before this gets used in anger, the user needs to give us the passphrase
        let ssh_keygen_output = Command::new("ssh-keygen")
            .current_dir(tempdir.path())
            .args(&[
                "-t",
                &key_type,
                "-m",
                &key_format,
                "-b",
                &self.component.bits.to_string(),
                "-C",
                &self.entity.name,
                "-f",
                &filename.to_string_lossy(),
                "-N",
                "\"\"",
            ])
            .output()
            .await?;
        event!(Level::DEBUG, ?ssh_keygen_output);
        error_on_failure(&ssh_keygen_output)?;

        let private_key = fs::read(&filename).await?;
        self.entity.private_key = String::from_utf8(private_key)?;

        let public_key_name = format!("{}.pub", filename.display());
        let public_key = fs::read(&public_key_name).await?;
        self.entity.public_key = String::from_utf8(public_key)?;
        event!(Level::DEBUG, ?self.entity.public_key);

        let fingerprint_output = Command::new("ssh-keygen")
            .current_dir(tempdir.path())
            .args(&["-l", "-f", &filename.to_string_lossy()])
            .output()
            .await?;
        event!(Level::DEBUG, ?fingerprint_output);
        error_on_failure(&fingerprint_output)?;
        self.entity.fingerprint = String::from_utf8(fingerprint_output.stdout)?;

        let bubble_babble_output = Command::new("ssh-keygen")
            .current_dir(tempdir.path())
            .args(&["-B", "-f", &filename.to_string_lossy()])
            .output()
            .await?;
        event!(Level::DEBUG, ?bubble_babble_output);
        error_on_failure(&bubble_babble_output)?;
        self.entity.bubble_babble = String::from_utf8(bubble_babble_output.stdout)?;

        let random_art_output = Command::new("ssh-keygen")
            .current_dir(tempdir.path())
            .args(&["-l", "-v", "-f", &filename.to_string_lossy()])
            .output()
            .await?;
        event!(Level::DEBUG, ?random_art_output);
        error_on_failure(&random_art_output)?;
        self.entity.random_art = String::from_utf8(random_art_output.stdout)?;

        tempdir.close()?;

        Ok(())
    }
}

fn error_on_failure(output: &std::process::Output) -> Result<()> {
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = output.status.code().unwrap_or(0);
        return Err(Error::SshKeyGenError(
            status,
            stdout.to_string(),
            stderr.to_string(),
        ));
    }
    Ok(())
}
