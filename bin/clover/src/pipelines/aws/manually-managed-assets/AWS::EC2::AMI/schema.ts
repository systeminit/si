import { CfSchema } from "../../schema.ts";
import { FuncSpecInfo } from "../../../../spec/funcs.ts";
import { ExpandedSchemaVariantSpec } from "../../../../spec/pkgs.ts";
import { addPropSuggestSource, addPropSuggestAsSourceFor } from "../../../../spec/props.ts";
import { createPropFinder } from "../../../generic/index.ts";

/**
 * AWS::EC2::AMI - Complete Definition
 *
 * This file contains both the schema and function specs for AWS::EC2::AMI.
 * Everything about this resource is in one place.
 */

// ============================================================================
// SCHEMA
// ============================================================================

export const schema: CfSchema = {
  typeName: "AWS::EC2::AMI",
  description:
    "Amazon Machine Image (AMI) query and validation resource. Allows querying for AMIs using filters and validates the selected AMI exists.",
  sourceUrl: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html",
  documentationUrl:
    "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html",
  properties: {
    // Domain properties (query parameters)
    ExecutableUsers: {
      type: "string",
      description:
        "Scopes the images by users with explicit launch permissions. Specify an Amazon Web Services account ID, `self` (the sender of the request), or `all` (public AMIs).\n\nIf you specify an Amazon Web Services account ID that is not your own, only AMIs shared with that specific Amazon Web Services account ID are returned. However, AMIs that are shared with the account's organization or organizational unit (OU) are not returned.",
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
    } as any,
    Owners: {
      type: "string",
      description:
        "Scopes the results to images with the specified owners. You can specify a combination of Amazon Web Services account IDs, `self`, `amazon`, `aws-backup-vault`, and `aws-marketplace`. If you omit this parameter, the results include all images for which you have launch permissions, regardless of ownership.",
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
    } as any,
    UseMostRecent: {
      type: "boolean",
      description: "Sort the results and return the most recent image.",
      default: true,
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
    } as any,
    Filters: {
      type: "array",
      description:
        "A list of filters to refine the image search - a full list of filters can be found on the [API Documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters)",
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
      itemName: "Filter", // Override default "FiltersItem" to prevent breaking changes
      items: {
        type: "object",
        properties: {
          Name: {
            type: "string",
            description:
              "The name of the filter. Filter names are case-sensitive.",
            docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
          } as any,
          Value: {
            type: "string",
            description:
              "The filter values. Filter values are case-sensitive. If you specify multiple values for a filter, the values are joined with an `OR`, and the request returns all results that match any of the specified values.",
            docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
          } as any,
        },
      },
    } as any,
    ImageId: {
      type: "string",
      description:
        "The image IDs. If specified, this will take precedence over the query created by using filters.",
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
    } as any,
    region: {
      type: "string",
      description: "The AWS region to query for AMIs",
      docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeImages.html#API_DescribeImages_RequestParameters",
    } as any,
    credential: {
      type: "string",
      description: "AWS credential for authentication",
    },
  },
  definitions: {},
  primaryIdentifier: ["/properties/ImageId"],
  readOnlyProperties: [],
  createOnlyProperties: [],
  writeOnlyProperties: [],
  handlers: {},
  // Explicit secret kind mapping - properties listed here become secrets
  secretKinds: {
    credential: "AWS Credential",
  },
} as const;

// ============================================================================
// SCHEMA CONFIGURATION
// ============================================================================

/**
 * Complete configuration for the AWS::EC2::AMI schema.
 * This includes functions, bindings, and metadata that extend the base schema.
 *
 * Function IDs must be stable - use SHA256 hash of a UUID:
 * - Generate: uuidgen | shasum -a 256
 */
export const config = {
  // Schema metadata - overrides defaults from provider config
  metadata: {
    displayName: "AMI Query",
    category: "AWS::EC2",
    color: "#FF9900",
    description:
      "Query and validate Amazon Machine Images (AMIs) using filters",
  },

  // Qualification functions
  qualification: {
    "Validate AMI Query": {
      id: "e8b9a8a41fd88e1a8cc3a1f3646af0a6dd4f7f578e0a7960b167cba28f5c4f4b",
      displayName: "Validate AMI Query",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::EC2::AMI/qualifications/qualificationAmiExists.ts",
      backendKind: "jsAttribute",
      responseType: "qualification",
    },
  } as const satisfies Record<string, FuncSpecInfo>,

  // Attribute functions to query for AMI IDs
  attribute: {
    "Query AMI ID": {
      id: "6e74c3c417eb8fd0d2ac05d838f98a848d150a4f7e2792b4e509ce522b1be1df",
      displayName: "Query AMI ID",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::EC2::AMI/attributes/awsEc2AmiQueryImageId.ts",
      backendKind: "jsAttribute",
      responseType: "string", // Returns a string (the ImageId)
    },
  } as const satisfies Record<string, FuncSpecInfo>,

  // Attribute function configuration - simplified!
  // Just specify which property to attach to and which properties to use as inputs
  attributeFunctions: (_variant: ExpandedSchemaVariantSpec) => {
    return {
      "Query AMI ID": {
        attachTo: "ImageId",
        inputs: ["region", "UseMostRecent", "Owners", "Filters", "ExecutableUsers"],
      },
    };
  },

  // Configure properties after variant creation
  configureProperties: (variant: ExpandedSchemaVariantSpec) => {
    // Use the generic property finder
    const findProp = createPropFinder(variant, "AWS::EC2::AMI");

    // Set up suggest sources
    const regionProp = findProp("region");
    addPropSuggestSource(regionProp, {
      schema: "Region",
      prop: "/domain/region",
    });

    const imageIdProp = findProp("ImageId");
    addPropSuggestAsSourceFor(imageIdProp, {
      schema: "AWS::EC2::Instance",
      prop: "/domain/ImageId",
    });

    // Note: credential secretKind and docLinks are configured in the schema
  },
};

export default {
  schema,
  config,
};
