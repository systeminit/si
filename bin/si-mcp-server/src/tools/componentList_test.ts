import { assertEquals } from "jsr:@std/assert";

// Stub environment variables before importing
Deno.env.set("SI_API_TOKEN", "test-token");
Deno.env.set("SI_WORKSPACE_ID", "test-workspace-id");

import { applyFilters } from "./componentList.ts";

// Test data
const testComponents = [
  {
    componentId: "comp-001",
    componentName: "vpc-prod-main",
    schemaName: "AWS::EC2::VPC",
  },
  {
    componentId: "comp-002",
    componentName: "vpc-dev-test",
    schemaName: "AWS::EC2::VPC",
  },
  {
    componentId: "comp-003",
    componentName: "bucket-prod-data",
    schemaName: "AWS::S3::Bucket",
  },
  {
    componentId: "comp-004",
    componentName: "instance-prod-web",
    schemaName: "AWS::EC2::Instance",
  },
  {
    componentId: "comp-005",
    componentName: "db-dev-mysql",
    schemaName: "AWS::RDS::DBInstance",
  },
];

Deno.test("applyFilters - returns all components when no filters provided", () => {
  const result = applyFilters(testComponents);
  assertEquals(result.length, 5);
  assertEquals(result, testComponents);
});

Deno.test("applyFilters - returns all components when empty filters provided", () => {
  const result = applyFilters(testComponents, { filterGroups: [] });
  assertEquals(result.length, 5);
  assertEquals(result, testComponents);
});

Deno.test("applyFilters - filters by componentName with single regex", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      regularExpressions: ["^vpc"],
    }],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 2);
  assertEquals(result[0].componentName, "vpc-prod-main");
  assertEquals(result[1].componentName, "vpc-dev-test");
});

Deno.test("applyFilters - filters by schemaName with multiple regex (OR logic)", () => {
  const filters = {
    filterGroups: [{
      responseField: "schemaName" as const,
      regularExpressions: ["EC2::VPC", "S3::Bucket"],
    }],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 3);
  // Should match VPCs and S3 bucket
  assertEquals(result.map((c) => c.schemaName).sort(), [
    "AWS::EC2::VPC",
    "AWS::EC2::VPC",
    "AWS::S3::Bucket",
  ]);
});

Deno.test("applyFilters - filters by componentId", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentId" as const,
      regularExpressions: ["comp-00[13]"],
    }],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 2);
  assertEquals(result[0].componentId, "comp-001");
  assertEquals(result[1].componentId, "comp-003");
});

Deno.test("applyFilters - single filter group with AND logic requires all regex patterns to match", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      logic: "AND" as const,
      regularExpressions: ["^vpc", "prod"],
    }],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 1);
  assertEquals(result[0].componentName, "vpc-prod-main");
});

Deno.test("applyFilters - AND logic returns empty when conditions cannot be met", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      logic: "AND" as const,
      regularExpressions: ["^vpc", "bucket"],
    }],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 0);
});

Deno.test("applyFilters - multiple filter groups with AND logic requires all groups to match", () => {
  const filters = {
    filterGroups: [
      {
        responseField: "componentName" as const,
        regularExpressions: ["prod"],
      },
      {
        responseField: "schemaName" as const,
        regularExpressions: ["EC2"],
      },
    ],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 2);
  // Should match vpc-prod-main and instance-prod-web
  assertEquals(result.map((c) => c.componentName).sort(), [
    "instance-prod-web",
    "vpc-prod-main",
  ]);
});

Deno.test("applyFilters - multiple filter groups with OR logic matches if any group matches", () => {
  const filters = {
    logic: "OR" as const,
    filterGroups: [
      {
        responseField: "componentName" as const,
        regularExpressions: ["^vpc"],
      },
      {
        responseField: "schemaName" as const,
        regularExpressions: ["S3::Bucket"],
      },
    ],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 3);
  // Should match both VPCs and the S3 bucket
  assertEquals(result.map((c) => c.componentName).sort(), [
    "bucket-prod-data",
    "vpc-dev-test",
    "vpc-prod-main",
  ]);
});

Deno.test("applyFilters - handles complex AND/OR combinations", () => {
  const filters = {
    logic: "OR" as const,
    filterGroups: [
      {
        responseField: "componentName" as const,
        logic: "AND" as const,
        regularExpressions: ["^vpc", "prod"],
      },
      {
        responseField: "schemaName" as const,
        logic: "OR" as const,
        regularExpressions: ["S3", "RDS"],
      },
    ],
  };

  const result = applyFilters(testComponents, filters);
  assertEquals(result.length, 3);
  // Should match: vpc-prod-main (first group), bucket-prod-data and db-dev-mysql (second group)
  assertEquals(result.map((c) => c.componentName).sort(), [
    "bucket-prod-data",
    "db-dev-mysql",
    "vpc-prod-main",
  ]);
});

Deno.test("applyFilters - handles invalid regex patterns gracefully", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      regularExpressions: ["[invalid", "^vpc"], // First regex is invalid
    }],
  };

  const result = applyFilters(testComponents, filters);
  // Should still match components with "^vpc" pattern
  assertEquals(result.length, 2);
  assertEquals(result[0].componentName, "vpc-prod-main");
  assertEquals(result[1].componentName, "vpc-dev-test");
});

Deno.test("applyFilters - handles all invalid regex patterns", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      regularExpressions: ["[invalid", "*broken"], // Both invalid
    }],
  };

  const result = applyFilters(testComponents, filters);
  // Should return no matches since all regex patterns are invalid
  assertEquals(result.length, 0);
});

Deno.test("applyFilters - handles invalid regex with AND logic", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      logic: "AND" as const,
      regularExpressions: ["[invalid", "^vpc"], // First regex is invalid
    }],
  };

  const result = applyFilters(testComponents, filters);
  // With AND logic, invalid regex (false) AND valid match = false
  assertEquals(result.length, 0);
});

Deno.test("applyFilters - works with empty component list", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      regularExpressions: ["^vpc"],
    }],
  };

  const result = applyFilters([], filters);
  assertEquals(result.length, 0);
});

Deno.test("applyFilters - works with empty regex array", () => {
  const filters = {
    filterGroups: [{
      responseField: "componentName" as const,
      regularExpressions: [],
    }],
  };

  const result = applyFilters(testComponents, filters);
  // Empty regex array with OR logic should match nothing
  assertEquals(result.length, 0);
});
