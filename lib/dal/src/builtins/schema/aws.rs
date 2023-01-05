use crate::builtins::schema::MigrationDriver;
use crate::{BuiltinsResult, DalContext};

mod core;
mod vpc;

// Reference: https://aws.amazon.com/trademark-guidelines/
const AWS_NODE_COLOR: i64 = 0xFF9900;

// Common documentation URLs
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const EC2_INSTANCE_TYPES_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-types.html";

impl MigrationDriver {
    pub async fn migrate_aws(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let ui_menu_category = "AWS";
        self.migrate_aws_core(ctx, ui_menu_category).await?;
        self.migrate_aws_vpc(ctx, ui_menu_category).await?;
        Ok(())
    }
}
