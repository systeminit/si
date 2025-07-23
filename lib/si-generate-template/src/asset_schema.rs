use askama::Template;

#[derive(Template, Debug)]
#[template(path = "asset_schema.ts")]
pub struct AssetSchema {
    aws: bool,
}

impl AssetSchema {
    pub fn new(aws: bool) -> AssetSchema {
        AssetSchema { aws }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_schema_without_aws() {
        let asset_schema = AssetSchema::new(false);
        let result = dbg!(
            asset_schema
                .render()
                .expect("failed to render asset schema with no aws resources")
        );
        assert!(
            !result.contains("AWS Credential"),
            "Should not render AWS Credential"
        );
    }

    #[test]
    fn asset_schema_with_aws() {
        let asset_schema = AssetSchema::new(true);
        let result = dbg!(
            asset_schema
                .render()
                .expect("failed to render asset schema with no aws resources")
        );
        assert!(
            result.contains("AWS Credential"),
            "Should render AWS Credential"
        );
        assert!(
            result.contains("Region"),
            "Should render domain/extra/Region"
        );
    }
}
