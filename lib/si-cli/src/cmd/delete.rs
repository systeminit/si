use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};

impl AppState {
    pub async fn delete(&self, keep_images: bool) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "delete-system"}),
        );
        invoke(self, self.is_preview(), keep_images).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, is_preview: bool, keep_images: bool) -> CliResult<()> {
    app.check(true).await?;
    app.stop().await?;

    if is_preview {
        println!("Deleted the following containers and associated images:");
    }

    for name in CONTAINER_NAMES.iter() {
        let container_name = format!("local-{0}-1", name);
        if is_preview {
            println!("{}", container_name);
            continue;
        }
        let container_summary = app
            .container_engine()
            .get_existing_container(container_name.clone())
            .await?;
        if let Some(container_summary) = container_summary {
            app.container_engine()
                .delete_container(
                    container_summary.id.unwrap().to_string(),
                    container_name.clone(),
                )
                .await?;

            if !keep_images {
                app.container_engine()
                    .cleanup_image(name.to_string())
                    .await?;
            }
        }
    }

    app.container_engine().delete_network().await?;

    Ok(())
}
