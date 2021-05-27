import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  WidgetSelectOptionsItems,
  ValidatorKind,
  //Arity,
} from "../../registryEntry";

import _ from "lodash";

// Taken from the output of az account list-locations on 2021-05-23
export const azureLocations: Record<string, any>[] = [
  {
    displayName: "East US",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus",
    metadata: {
      geographyGroup: "US",
      latitude: "37.3719",
      longitude: "-79.8164",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus",
          name: "westus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Virginia",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "eastus",
    regionalDisplayName: "(US) East US",
    subscriptionId: null,
  },
  {
    displayName: "East US 2",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus2",
    metadata: {
      geographyGroup: "US",
      latitude: "36.6681",
      longitude: "-78.3889",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centralus",
          name: "centralus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Virginia",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "eastus2",
    regionalDisplayName: "(US) East US 2",
    subscriptionId: null,
  },
  {
    displayName: "South Central US",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southcentralus",
    metadata: {
      geographyGroup: "US",
      latitude: "29.4167",
      longitude: "-98.5",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/northcentralus",
          name: "northcentralus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Texas",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "southcentralus",
    regionalDisplayName: "(US) South Central US",
    subscriptionId: null,
  },
  {
    displayName: "West US 2",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus2",
    metadata: {
      geographyGroup: "US",
      latitude: "47.233",
      longitude: "-119.852",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westcentralus",
          name: "westcentralus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Washington",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "westus2",
    regionalDisplayName: "(US) West US 2",
    subscriptionId: null,
  },
  {
    displayName: "West US 3",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus3",
    metadata: {
      geographyGroup: "US",
      latitude: "33.448376",
      longitude: "-112.074036",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus",
          name: "eastus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Phoenix",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "westus3",
    regionalDisplayName: "(US) West US 3",
    subscriptionId: null,
  },
  {
    displayName: "Australia East",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiaeast",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "-33.86",
      longitude: "151.2094",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiasoutheast",
          name: "australiasoutheast",
          subscriptionId: null,
        },
      ],
      physicalLocation: "New South Wales",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "australiaeast",
    regionalDisplayName: "(Asia Pacific) Australia East",
    subscriptionId: null,
  },
  {
    displayName: "Southeast Asia",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southeastasia",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "1.283",
      longitude: "103.833",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastasia",
          name: "eastasia",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Singapore",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "southeastasia",
    regionalDisplayName: "(Asia Pacific) Southeast Asia",
    subscriptionId: null,
  },
  {
    displayName: "North Europe",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/northeurope",
    metadata: {
      geographyGroup: "Europe",
      latitude: "53.3478",
      longitude: "-6.2597",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westeurope",
          name: "westeurope",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Ireland",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "northeurope",
    regionalDisplayName: "(Europe) North Europe",
    subscriptionId: null,
  },
  {
    displayName: "UK South",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uksouth",
    metadata: {
      geographyGroup: "Europe",
      latitude: "50.941",
      longitude: "-0.799",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/ukwest",
          name: "ukwest",
          subscriptionId: null,
        },
      ],
      physicalLocation: "London",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "uksouth",
    regionalDisplayName: "(Europe) UK South",
    subscriptionId: null,
  },
  {
    displayName: "West Europe",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westeurope",
    metadata: {
      geographyGroup: "Europe",
      latitude: "52.3667",
      longitude: "4.9",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/northeurope",
          name: "northeurope",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Netherlands",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "westeurope",
    regionalDisplayName: "(Europe) West Europe",
    subscriptionId: null,
  },
  {
    displayName: "Central US",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centralus",
    metadata: {
      geographyGroup: "US",
      latitude: "41.5908",
      longitude: "-93.6208",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus2",
          name: "eastus2",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Iowa",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "centralus",
    regionalDisplayName: "(US) Central US",
    subscriptionId: null,
  },
  {
    displayName: "North Central US",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/northcentralus",
    metadata: {
      geographyGroup: "US",
      latitude: "41.8819",
      longitude: "-87.6278",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southcentralus",
          name: "southcentralus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Illinois",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "northcentralus",
    regionalDisplayName: "(US) North Central US",
    subscriptionId: null,
  },
  {
    displayName: "West US",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus",
    metadata: {
      geographyGroup: "US",
      latitude: "37.783",
      longitude: "-122.417",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus",
          name: "eastus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "California",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "westus",
    regionalDisplayName: "(US) West US",
    subscriptionId: null,
  },
  {
    displayName: "South Africa North",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southafricanorth",
    metadata: {
      geographyGroup: "Africa",
      latitude: "-25.731340",
      longitude: "28.218370",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southafricawest",
          name: "southafricawest",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Johannesburg",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "southafricanorth",
    regionalDisplayName: "(Africa) South Africa North",
    subscriptionId: null,
  },
  {
    displayName: "Central India",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centralindia",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "18.5822",
      longitude: "73.9197",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southindia",
          name: "southindia",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Pune",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "centralindia",
    regionalDisplayName: "(Asia Pacific) Central India",
    subscriptionId: null,
  },
  {
    displayName: "East Asia",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastasia",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "22.267",
      longitude: "114.188",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southeastasia",
          name: "southeastasia",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Hong Kong",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "eastasia",
    regionalDisplayName: "(Asia Pacific) East Asia",
    subscriptionId: null,
  },
  {
    displayName: "Japan East",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/japaneast",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "35.68",
      longitude: "139.77",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/japanwest",
          name: "japanwest",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Tokyo, Saitama",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "japaneast",
    regionalDisplayName: "(Asia Pacific) Japan East",
    subscriptionId: null,
  },
  {
    displayName: "Jio India West",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/jioindiawest",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "22.470701",
      longitude: "70.05773",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/jioindiacentral",
          name: "jioindiacentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Jamnagar",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "jioindiawest",
    regionalDisplayName: "(Asia Pacific) Jio India West",
    subscriptionId: null,
  },
  {
    displayName: "Korea Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/koreacentral",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "37.5665",
      longitude: "126.9780",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/koreasouth",
          name: "koreasouth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Seoul",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "koreacentral",
    regionalDisplayName: "(Asia Pacific) Korea Central",
    subscriptionId: null,
  },
  {
    displayName: "Canada Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/canadacentral",
    metadata: {
      geographyGroup: "Canada",
      latitude: "43.653",
      longitude: "-79.383",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/canadaeast",
          name: "canadaeast",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Toronto",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "canadacentral",
    regionalDisplayName: "(Canada) Canada Central",
    subscriptionId: null,
  },
  {
    displayName: "France Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/francecentral",
    metadata: {
      geographyGroup: "Europe",
      latitude: "46.3772",
      longitude: "2.3730",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/francesouth",
          name: "francesouth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Paris",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "francecentral",
    regionalDisplayName: "(Europe) France Central",
    subscriptionId: null,
  },
  {
    displayName: "Germany West Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/germanywestcentral",
    metadata: {
      geographyGroup: "Europe",
      latitude: "50.110924",
      longitude: "8.682127",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/germanynorth",
          name: "germanynorth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Frankfurt",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "germanywestcentral",
    regionalDisplayName: "(Europe) Germany West Central",
    subscriptionId: null,
  },
  {
    displayName: "Norway East",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/norwayeast",
    metadata: {
      geographyGroup: "Europe",
      latitude: "59.913868",
      longitude: "10.752245",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/norwaywest",
          name: "norwaywest",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Norway",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "norwayeast",
    regionalDisplayName: "(Europe) Norway East",
    subscriptionId: null,
  },
  {
    displayName: "Switzerland North",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/switzerlandnorth",
    metadata: {
      geographyGroup: "Europe",
      latitude: "47.451542",
      longitude: "8.564572",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/switzerlandwest",
          name: "switzerlandwest",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Zurich",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "switzerlandnorth",
    regionalDisplayName: "(Europe) Switzerland North",
    subscriptionId: null,
  },
  {
    displayName: "UAE North",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uaenorth",
    metadata: {
      geographyGroup: "Middle East",
      latitude: "25.266666",
      longitude: "55.316666",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uaecentral",
          name: "uaecentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Dubai",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "uaenorth",
    regionalDisplayName: "(Middle East) UAE North",
    subscriptionId: null,
  },
  {
    displayName: "Brazil South",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/brazilsouth",
    metadata: {
      geographyGroup: "South America",
      latitude: "-23.55",
      longitude: "-46.633",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southcentralus",
          name: "southcentralus",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Sao Paulo State",
      regionCategory: "Recommended",
      regionType: "Physical",
    },
    name: "brazilsouth",
    regionalDisplayName: "(South America) Brazil South",
    subscriptionId: null,
  },
  {
    displayName: "Central US (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centralusstage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "centralusstage",
    regionalDisplayName: "(US) Central US (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "East US (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastusstage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "eastusstage",
    regionalDisplayName: "(US) East US (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "East US 2 (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus2stage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "eastus2stage",
    regionalDisplayName: "(US) East US 2 (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "North Central US (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/northcentralusstage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "northcentralusstage",
    regionalDisplayName: "(US) North Central US (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "South Central US (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southcentralusstage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "southcentralusstage",
    regionalDisplayName: "(US) South Central US (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "West US (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westusstage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "westusstage",
    regionalDisplayName: "(US) West US (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "West US 2 (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus2stage",
    metadata: {
      geographyGroup: "US",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "westus2stage",
    regionalDisplayName: "(US) West US 2 (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "Asia",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/asia",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "asia",
    regionalDisplayName: "Asia",
    subscriptionId: null,
  },
  {
    displayName: "Asia Pacific",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/asiapacific",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "asiapacific",
    regionalDisplayName: "Asia Pacific",
    subscriptionId: null,
  },
  {
    displayName: "Australia",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australia",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "australia",
    regionalDisplayName: "Australia",
    subscriptionId: null,
  },
  {
    displayName: "Brazil",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/brazil",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "brazil",
    regionalDisplayName: "Brazil",
    subscriptionId: null,
  },
  {
    displayName: "Canada",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/canada",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "canada",
    regionalDisplayName: "Canada",
    subscriptionId: null,
  },
  {
    displayName: "Europe",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/europe",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "europe",
    regionalDisplayName: "Europe",
    subscriptionId: null,
  },
  {
    displayName: "Global",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/global",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "global",
    regionalDisplayName: "Global",
    subscriptionId: null,
  },
  {
    displayName: "India",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/india",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "india",
    regionalDisplayName: "India",
    subscriptionId: null,
  },
  {
    displayName: "Japan",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/japan",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "japan",
    regionalDisplayName: "Japan",
    subscriptionId: null,
  },
  {
    displayName: "United Kingdom",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uk",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "uk",
    regionalDisplayName: "United Kingdom",
    subscriptionId: null,
  },
  {
    displayName: "United States",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/unitedstates",
    metadata: {
      geographyGroup: null,
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "unitedstates",
    regionalDisplayName: "United States",
    subscriptionId: null,
  },
  {
    displayName: "East Asia (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastasiastage",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "eastasiastage",
    regionalDisplayName: "(Asia Pacific) East Asia (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "Southeast Asia (Stage)",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southeastasiastage",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: null,
      longitude: null,
      pairedRegion: null,
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Logical",
    },
    name: "southeastasiastage",
    regionalDisplayName: "(Asia Pacific) Southeast Asia (Stage)",
    subscriptionId: null,
  },
  {
    displayName: "Central US EUAP",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centraluseuap",
    metadata: {
      geographyGroup: "US",
      latitude: "41.5908",
      longitude: "-93.6208",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus2euap",
          name: "eastus2euap",
          subscriptionId: null,
        },
      ],
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "centraluseuap",
    regionalDisplayName: "(US) Central US EUAP",
    subscriptionId: null,
  },
  {
    displayName: "East US 2 EUAP",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/eastus2euap",
    metadata: {
      geographyGroup: "US",
      latitude: "36.6681",
      longitude: "-78.3889",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centraluseuap",
          name: "centraluseuap",
          subscriptionId: null,
        },
      ],
      physicalLocation: null,
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "eastus2euap",
    regionalDisplayName: "(US) East US 2 EUAP",
    subscriptionId: null,
  },
  {
    displayName: "West Central US",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westcentralus",
    metadata: {
      geographyGroup: "US",
      latitude: "40.890",
      longitude: "-110.234",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westus2",
          name: "westus2",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Wyoming",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "westcentralus",
    regionalDisplayName: "(US) West Central US",
    subscriptionId: null,
  },
  {
    displayName: "South Africa West",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southafricawest",
    metadata: {
      geographyGroup: "Africa",
      latitude: "-34.075691",
      longitude: "18.843266",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southafricanorth",
          name: "southafricanorth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Cape Town",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "southafricawest",
    regionalDisplayName: "(Africa) South Africa West",
    subscriptionId: null,
  },
  {
    displayName: "Australia Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiacentral",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "-35.3075",
      longitude: "149.1244",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiacentral",
          name: "australiacentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Canberra",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "australiacentral",
    regionalDisplayName: "(Asia Pacific) Australia Central",
    subscriptionId: null,
  },
  {
    displayName: "Australia Central 2",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiacentral2",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "-35.3075",
      longitude: "149.1244",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiacentral2",
          name: "australiacentral2",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Canberra",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "australiacentral2",
    regionalDisplayName: "(Asia Pacific) Australia Central 2",
    subscriptionId: null,
  },
  {
    displayName: "Australia Southeast",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiasoutheast",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "-37.8136",
      longitude: "144.9631",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/australiaeast",
          name: "australiaeast",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Victoria",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "australiasoutheast",
    regionalDisplayName: "(Asia Pacific) Australia Southeast",
    subscriptionId: null,
  },
  {
    displayName: "Japan West",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/japanwest",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "34.6939",
      longitude: "135.5022",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/japaneast",
          name: "japaneast",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Osaka",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "japanwest",
    regionalDisplayName: "(Asia Pacific) Japan West",
    subscriptionId: null,
  },
  {
    displayName: "Korea South",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/koreasouth",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "35.1796",
      longitude: "129.0756",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/koreacentral",
          name: "koreacentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Busan",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "koreasouth",
    regionalDisplayName: "(Asia Pacific) Korea South",
    subscriptionId: null,
  },
  {
    displayName: "South India",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southindia",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "12.9822",
      longitude: "80.1636",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/centralindia",
          name: "centralindia",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Chennai",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "southindia",
    regionalDisplayName: "(Asia Pacific) South India",
    subscriptionId: null,
  },
  {
    displayName: "West India",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/westindia",
    metadata: {
      geographyGroup: "Asia Pacific",
      latitude: "19.088",
      longitude: "72.868",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/southindia",
          name: "southindia",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Mumbai",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "westindia",
    regionalDisplayName: "(Asia Pacific) West India",
    subscriptionId: null,
  },
  {
    displayName: "Canada East",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/canadaeast",
    metadata: {
      geographyGroup: "Canada",
      latitude: "46.817",
      longitude: "-71.217",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/canadacentral",
          name: "canadacentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Quebec",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "canadaeast",
    regionalDisplayName: "(Canada) Canada East",
    subscriptionId: null,
  },
  {
    displayName: "France South",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/francesouth",
    metadata: {
      geographyGroup: "Europe",
      latitude: "43.8345",
      longitude: "2.1972",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/francecentral",
          name: "francecentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Marseille",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "francesouth",
    regionalDisplayName: "(Europe) France South",
    subscriptionId: null,
  },
  {
    displayName: "Germany North",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/germanynorth",
    metadata: {
      geographyGroup: "Europe",
      latitude: "53.073635",
      longitude: "8.806422",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/germanywestcentral",
          name: "germanywestcentral",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Berlin",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "germanynorth",
    regionalDisplayName: "(Europe) Germany North",
    subscriptionId: null,
  },
  {
    displayName: "Norway West",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/norwaywest",
    metadata: {
      geographyGroup: "Europe",
      latitude: "58.969975",
      longitude: "5.733107",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/norwayeast",
          name: "norwayeast",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Norway",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "norwaywest",
    regionalDisplayName: "(Europe) Norway West",
    subscriptionId: null,
  },
  {
    displayName: "Switzerland West",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/switzerlandwest",
    metadata: {
      geographyGroup: "Europe",
      latitude: "46.204391",
      longitude: "6.143158",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/switzerlandnorth",
          name: "switzerlandnorth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Geneva",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "switzerlandwest",
    regionalDisplayName: "(Europe) Switzerland West",
    subscriptionId: null,
  },
  {
    displayName: "UK West",
    id: "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/ukwest",
    metadata: {
      geographyGroup: "Europe",
      latitude: "53.427",
      longitude: "-3.084",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uksouth",
          name: "uksouth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Cardiff",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "ukwest",
    regionalDisplayName: "(Europe) UK West",
    subscriptionId: null,
  },
  {
    displayName: "UAE Central",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uaecentral",
    metadata: {
      geographyGroup: "Middle East",
      latitude: "24.466667",
      longitude: "54.366669",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/uaenorth",
          name: "uaenorth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Abu Dhabi",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "uaecentral",
    regionalDisplayName: "(Middle East) UAE Central",
    subscriptionId: null,
  },
  {
    displayName: "Brazil Southeast",
    id:
      "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/brazilsoutheast",
    metadata: {
      geographyGroup: "South America",
      latitude: "-22.90278",
      longitude: "-43.2075",
      pairedRegion: [
        {
          id:
            "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/locations/brazilsouth",
          name: "brazilsouth",
          subscriptionId: null,
        },
      ],
      physicalLocation: "Rio",
      regionCategory: "Other",
      regionType: "Physical",
    },
    name: "brazilsoutheast",
    regionalDisplayName: "(South America) Brazil Southeast",
    subscriptionId: null,
  },
];

export function generateLabels(): WidgetSelectOptionsItems {
  const items = _.map(azureLocations, (l) => {
    return { label: `${l.displayName} (${l.name})`, value: l.name };
  });
  return { items };
}

const azureLocation: RegistryEntry = {
  entityType: "azureLocation",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "location",
        menuCategory: ["azure"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["cloudProvider"],
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "location",
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

export default azureLocation;
