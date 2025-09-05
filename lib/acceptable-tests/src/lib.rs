#[cfg(test)]
mod tests {
    use acceptable::{
        AllVersions,
        Container,
        CurrentContainer,
        DeserializeContainer,
        IntoContainer,
        RequestId,
        SerializeContainer,
        SupportsContainers as _,
        UpgradeError,
        Versioned,
    };
    use serde::{
        Deserialize,
        Serialize,
    };

    #[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
    #[serde(rename_all = "camelCase")]
    #[acceptable(version = 1)]
    struct CoolRequestV1 {
        id: RequestId,
    }

    #[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
    #[serde(rename_all = "camelCase")]
    #[acceptable(version = 2)]
    struct CoolRequestV2 {
        id: RequestId,
    }

    impl TryFrom<CoolRequestV1> for CoolRequestV2 {
        type Error = UpgradeError;

        fn try_from(value: CoolRequestV1) -> Result<Self, Self::Error> {
            Ok(Self { id: value.id })
        }
    }

    #[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
    #[serde(rename_all = "camelCase")]
    #[acceptable(version = 3)]
    struct CoolRequestV3 {
        id: RequestId,
        name: String,
    }

    impl TryFrom<CoolRequestV2> for CoolRequestV3 {
        type Error = UpgradeError;

        fn try_from(value: CoolRequestV2) -> Result<Self, Self::Error> {
            Ok(Self {
                id: value.id,
                name: "<unknown>".to_string(),
            })
        }
    }

    #[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
    #[serde(rename_all = "camelCase")]
    #[acceptable(version = 4)]
    struct CoolRequestV4 {
        id: RequestId,
        name: String,
        organization: Option<String>,
    }

    impl TryFrom<CoolRequestV3> for CoolRequestV4 {
        type Error = UpgradeError;

        fn try_from(value: CoolRequestV3) -> Result<Self, Self::Error> {
            Ok(Self {
                id: value.id,
                name: value.name,
                organization: None,
            })
        }
    }

    #[derive(AllVersions, CurrentContainer, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    enum CoolRequestAllVersions {
        V1(CoolRequestV1),
        V2(CoolRequestV2),
        V3(CoolRequestV3),
        #[acceptable(current)]
        V4(CoolRequestV4),
    }

    impl IntoContainer for CoolRequestAllVersions {
        type Container = CoolRequest;

        fn into_container(mut self) -> Result<Self::Container, UpgradeError> {
            loop {
                match self {
                    Self::V1(inner) => self = Self::V2(CoolRequestV2::try_from(inner)?),
                    Self::V2(inner) => self = Self::V3(CoolRequestV3::try_from(inner)?),
                    Self::V3(inner) => self = Self::V4(CoolRequestV4::try_from(inner)?),
                    Self::V4(inner) => return Ok(Self::Container::new(inner)),
                }
            }
        }
    }

    #[test]
    fn derived_types() {
        assert!(CoolRequest::is_message_type_supported("CoolRequest"));
        assert!(!CoolRequest::is_message_type_supported("Nope"));

        assert!(CoolRequest::is_content_type_supported("application/cbor"));
        assert!(!CoolRequest::is_content_type_supported("text/plain"));

        assert!(CoolRequest::is_message_version_supported(1));
        assert!(CoolRequest::is_message_version_supported(2));
        assert!(CoolRequest::is_message_version_supported(3));
        assert!(CoolRequest::is_message_version_supported(4));

        assert!(!CoolRequest::is_message_version_supported(0));
        assert!(!CoolRequest::is_message_version_supported(5));
        assert!(!CoolRequest::is_message_version_supported(42));

        let older_request = CoolRequestAllVersions::V2(CoolRequestV2 {
            id: RequestId::new(),
        });

        // Older version is queryable
        assert_eq!(older_request.version(), 2);

        dbg!(&older_request);

        // Older version can be upgraded into container type
        let mut request = older_request.into_container().expect("failed to upgrade");

        // Current version container type is queryable
        assert_eq!(request.version(), 4);
        assert_eq!(CoolRequest::message_version(), 4);
        // Current version has upgraded logic data
        assert_eq!(request.organization, None);

        dbg!(&request);

        // Current version can be mutated
        request.organization = Some("acme".to_string());
        assert_eq!(request.organization, Some("acme".to_string()));

        dbg!(&request);

        // Current container type can serialize
        let bytes = request.to_json_vec().expect("failed to serialize");
        let json_s = String::from_utf8(bytes.clone()).expect("failed to decode utf8 string");

        dbg!(json_s);

        // Serialized bytes can be deserialized via `AllVersions` helper back into current
        // container type
        let de_request = CoolRequest::from_json_slice(&bytes).expect("failed to deserialize");

        dbg!(&de_request);

        assert_eq!(request, de_request);
    }
}
