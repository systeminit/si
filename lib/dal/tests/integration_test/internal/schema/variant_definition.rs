use dal::{
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
    },
    DalContext,
};
use dal_test::test;

const VARIANT_DEFINITION_METADATA_JSON: &str = r#"{
  "link": "https://coreos.github.io/butane/config-fcos-v1_4/",
  "name": "OingoBoingo",
  "category": "NewWave", 
  "componentKind": "standard",
  "color": "DEADAF",
  "componentType": "component"
}
"#;

const VARIANT_DEFINITION_JSON: &str = r#"{
    "docLinks": {
        "default": "http://www.boingo.org/",
        "midi": "http://www.boingo.org/MIDI.html"
    },
    "props": [
        {
            "name": "oingo",
            "kind": "boolean", 
            "docLinkRef": "default"
        },
        {
            "name": "boingo",
            "kind": "array",
            "docLinkRef": "midi",
            "entry":  {
                "name": "boingoElem",
                "kind": "string"
            }
        },
        {
            "name": "deadmansparty",
            "kind": "integer" 
        },
        {
            "name": "weirdscience",
            "kind": "object", 
            "children": [
                {
                    "name": "mycreation",
                    "kind": "string" 
                },
                {
                    "name": "isitreal",
                    "kind": "string" 
                }
            ] 
        }
    ],
    "inputSockets": [
        {
            "name": "diagrams and charts"
        }
    ],
    "outputSockets": [
        {
            "name": "mending broken hearts"
        }
    ]
}
"#;

#[test]
async fn variant_definition_from_json(ctx: &DalContext) {
    let metadata: SchemaVariantDefinitionMetadataJson =
        serde_json::from_str(VARIANT_DEFINITION_METADATA_JSON)
            .expect("could not deserialize metadata");
    let definition: SchemaVariantDefinitionJson =
        serde_json::from_str(VARIANT_DEFINITION_JSON).expect("could not deserialize definition");

    let variant_definition = SchemaVariantDefinition::new_from_structs(ctx, metadata, definition)
        .await
        .expect("could not create variant definition from json");

    let _metadata_from_db_row: SchemaVariantDefinitionMetadataJson = (&variant_definition).into();
    let _definition_from_db_row: SchemaVariantDefinitionJson = variant_definition
        .try_into()
        .expect("Could not deserialize db row definition");
}
