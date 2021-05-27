import {
  RegistryEntry,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  WidgetSelectOptionsItems,
  //Arity,
} from "../../registryEntry";

import _ from "lodash";

// This list came from https://github.com/jsonmaur/aws-regions - covered under the MIT
// license.
const regions = [
  {
    name: "N. Virginia",
    full_name: "US East (N. Virginia)",
    code: "us-east-1",
    public: true,
    zones: [
      "us-east-1a",
      "us-east-1b",
      "us-east-1c",
      "us-east-1d",
      "us-east-1e",
      "us-east-1f",
    ],
  },
  {
    name: "Ohio",
    full_name: "US East (Ohio)",
    code: "us-east-2",
    public: true,
    zones: ["us-east-2a", "us-east-2b", "us-east-2c"],
  },
  {
    name: "N. California",
    full_name: "US West (N. California)",
    code: "us-west-1",
    public: true,
    zone_limit: 2,
    zones: ["us-west-1a", "us-west-1b", "us-west-1c"],
  },
  {
    name: "Oregon",
    full_name: "US West (Oregon)",
    code: "us-west-2",
    public: true,
    zones: ["us-west-2a", "us-west-2b", "us-west-2c", "us-west-2d"],
  },
  {
    name: "GovCloud West",
    full_name: "AWS GovCloud (US)",
    code: "us-gov-west-1",
    public: false,
    zones: ["us-gov-west-1a", "us-gov-west-1b", "us-gov-west-1c"],
  },
  {
    name: "GovCloud East",
    full_name: "AWS GovCloud (US-East)",
    code: "us-gov-east-1",
    public: false,
    zones: ["us-gov-east-1a", "us-gov-east-1b", "us-gov-east-1c"],
  },
  {
    name: "Canada",
    full_name: "Canada (Central)",
    code: "ca-central-1",
    public: true,
    zones: ["ca-central-1a", "ca-central-1b", "ca-central-1c"],
  },
  {
    name: "Stockholm",
    full_name: "EU (Stockholm)",
    code: "eu-north-1",
    public: true,
    zones: ["eu-north-1a", "eu-north-1b", "eu-north-1c"],
  },
  {
    name: "Ireland",
    full_name: "EU (Ireland)",
    code: "eu-west-1",
    public: true,
    zones: ["eu-west-1a", "eu-west-1b", "eu-west-1c"],
  },
  {
    name: "London",
    full_name: "EU (London)",
    code: "eu-west-2",
    public: true,
    zones: ["eu-west-2a", "eu-west-2b", "eu-west-2c"],
  },
  {
    name: "Paris",
    full_name: "EU (Paris)",
    code: "eu-west-3",
    public: true,
    zones: ["eu-west-3a", "eu-west-3b", "eu-west-3c"],
  },
  {
    name: "Frankfurt",
    full_name: "EU (Frankfurt)",
    code: "eu-central-1",
    public: true,
    zones: ["eu-central-1a", "eu-central-1b", "eu-central-1c"],
  },
  {
    name: "Milan",
    full_name: "EU (Milan)",
    code: "eu-south-1",
    public: true,
    zones: ["eu-south-1a", "eu-south-1b", "eu-south-1c"],
  },
  {
    name: "Cape Town",
    full_name: "Africa (Cape Town)",
    code: "af-south-1",
    public: true,
    zones: ["af-south-1a", "af-south-1b", "af-south-1c"],
  },
  {
    name: "Tokyo",
    full_name: "Asia Pacific (Tokyo)",
    code: "ap-northeast-1",
    public: true,
    zone_limit: 3,
    zones: [
      "ap-northeast-1a",
      "ap-northeast-1b",
      "ap-northeast-1c",
      "ap-northeast-1d",
    ],
  },
  {
    name: "Seoul",
    full_name: "Asia Pacific (Seoul)",
    code: "ap-northeast-2",
    public: true,
    zones: ["ap-northeast-2a", "ap-northeast-2b", "ap-northeast-2c"],
  },
  {
    name: "Osaka",
    full_name: "Asia Pacific (Osaka-Local)",
    code: "ap-northeast-3",
    public: false,
    zones: ["ap-northeast-3a"],
  },
  {
    name: "Singapore",
    full_name: "Asia Pacific (Singapore)",
    code: "ap-southeast-1",
    public: true,
    zones: ["ap-southeast-1a", "ap-southeast-1b", "ap-southeast-1c"],
  },
  {
    name: "Sydney",
    full_name: "Asia Pacific (Sydney)",
    code: "ap-southeast-2",
    public: true,
    zones: ["ap-southeast-2a", "ap-southeast-2b", "ap-southeast-2c"],
  },
  {
    name: "Hong Kong",
    full_name: "Asia Pacific (Hong Kong)",
    code: "ap-east-1",
    public: true,
    zones: ["ap-east-1a", "ap-east-1b", "ap-east-1c"],
  },
  {
    name: "Mumbai",
    full_name: "Asia Pacific (Mumbai)",
    code: "ap-south-1",
    public: true,
    zones: ["ap-south-1a", "ap-south-1b", "ap-south-1c"],
  },
  {
    name: "São Paulo",
    full_name: "South America (São Paulo)",
    code: "sa-east-1",
    public: true,
    zone_limit: 2,
    zones: ["sa-east-1a", "sa-east-1b", "sa-east-1c"],
  },
  {
    name: "Bahrain",
    full_name: "Middle East (Bahrain)",
    code: "me-south-1",
    public: true,
    zones: ["me-south-1a", "me-south-1b", "me-south-1c"],
  },
  {
    name: "Beijing",
    full_name: "China (Beijing)",
    code: "cn-north-1",
    public: false,
    zones: ["cn-north-1a", "cn-north-1b"],
  },
  {
    name: "Ningxia",
    full_name: "China (Ningxia)",
    code: "cn-northwest-1",
    public: false,
    zones: ["cn-northwest-1a", "cn-northwest-1b", "cn-northwest-1c"],
  },
];

function generateLabels(): WidgetSelectOptionsItems {
  const items = _.map(_.filter(regions, "public"), (r) => {
    return { label: `${r.full_name} (${r.code})`, value: r.code };
  });
  return { items };
}

const awsRegion: RegistryEntry = {
  entityType: "awsRegion",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "region",
        menuCategory: ["aws"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["cloudProvider"],
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "region",
      widget: {
        name: "select",
        options: generateLabels(),
      },
      validation: [
        {
          kind: ValidatorKind.Required,
        },
      ],
    },
  ],
};

export default awsRegion;
