import { assertObjectMatch } from "@std/assert/object-match";
import { assertArrayIncludes, assertEquals } from "@std/assert";
import layout from "../../src/sandbox/layout.ts";
import { converge } from "../../src/sandbox/template.ts";

Deno.test(function convergeHandlesCreate() {
  const thisComponent = {
    "kind": "TemplateTest",
    "properties": {
      "si": {
        "name": "template test",
        "type": "component",
        "color": "#00ffff",
      },
      "resource_value": {},
    },
    "geometry": {
      "tt": {
        "x": -18,
        "y": -488,
        "width": 0,
        "height": 0,
      },
    },
  };

  const specs = [];

  const vpcSpec = {
    "kind": "VPC",
    "properties": {
      "si": {
        "name": "vpc",
        "type": "configurationFrameDown",
        "color": "#ff9900",
      },
      "domain": {
        "CidrBlock": "10.0.0.0/24",
        "EnableDnsHostnames": true,
        "EnableDnsResolution": false,
        "awsResourceType": "vpc",
        "tags": {},
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 500.0,
      "width": 489.0,
      "height": 660.0,
    },
    "connect": null,
    "parent": null,
  };
  specs.push(vpcSpec);

  const subnet1Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet1",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1b",
        "IsPublic": true,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 555.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet1Spec);

  const subnet2Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet2",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1a",
        "IsPublic": false,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 56.0,
      "y": 804.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet2Spec);

  const expectedUpdates = {
    "status": "ok",
    "message": "Updated Components",
    "ops": {
      "create": {
        "vpc": {
          "kind": "VPC",
          "properties": {
            "si": {
              "name": "vpc",
              "type": "configurationFrameDown",
              "color": "#ff9900",
            },
            "domain": {
              "CidrBlock": "10.0.0.0/24",
              "EnableDnsHostnames": true,
              "EnableDnsResolution": false,
              "awsResourceType": "vpc",
              "tags": {},
            },
          },
          "geometry": {
            "x": 50,
            "y": 500,
            "width": 489,
            "height": 660,
          },
          "connect": null,
          "parent": null,
        },
        "subnet1": {
          "kind": "Subnet",
          "properties": {
            "si": {
              "name": "subnet1",
              "type": "component",
              "color": "#ff9900",
            },
            "domain": {
              "AvailabilityZone": "us-east-1b",
              "IsPublic": true,
              "Tags": {},
              "awsResourceType": "subnet",
            },
          },
          "geometry": {
            "x": 50,
            "y": 555,
            "width": null,
            "height": null,
          },
          "connect": null,
          "parent": "vpc",
        },
        "subnet2": {
          "kind": "Subnet",
          "properties": {
            "si": {
              "name": "subnet2",
              "type": "component",
              "color": "#ff9900",
            },
            "domain": {
              "AvailabilityZone": "us-east-1a",
              "IsPublic": false,
              "Tags": {},
              "awsResourceType": "subnet",
            },
          },
          "geometry": {
            "x": 56,
            "y": 804,
            "width": null,
            "height": null,
          },
          "connect": null,
          "parent": "vpc",
        },
      },
    },
  };
  const result = converge("test", thisComponent, {}, specs as any);
  assertObjectMatch(result, expectedUpdates);
});

Deno.test(function convergeHandlesNoOpUpdate() {
  const thisComponent = {
    "kind": "TemplateTest",
    "properties": {
      "si": {
        "name": "template test",
        "type": "component",
        "color": "#00ffff",
      },
      "resource_value": {},
    },
    "geometry": {
      "tt": {
        "x": -18,
        "y": -488,
        "width": 0,
        "height": 0,
      },
    },
  };

  const components = {
    "01JJT8A42J83RHBQ44N6T75YFR": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet2",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1a",
          "IsPublic": false,
          "Tags": {
            "Name": "subnet2",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1a",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet2"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 38,
          "y": 316,
          "width": null,
          "height": null,
        },
      },
    },
    "01JJT8A41VFT17AJMTSWE50BT2": {
      "kind": "VPC",
      "properties": {
        "si": {
          "name": "vpc",
          "type": "configurationFrameDown",
          "color": "#ff9900",
        },
        "domain": {
          "CidrBlock": "10.0.0.0/24",
          "EnableDnsHostnames": true,
          "EnableDnsResolution": false,
          "awsResourceType": "vpc",
          "tags": {
            "Name": "vpc",
          },
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsVpcJSON": {
            "code":
              '{\n\t"CidrBlock": "10.0.0.0/24",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "vpc",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "vpc"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsVpcCanCreate": {
            "result": "failure",
            "message": "no Region available",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 12,
          "width": 489,
          "height": 660,
        },
      },
    },
    "01JJT8A428SCERDG35B106PB77": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet1",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1b",
          "IsPublic": true,
          "Tags": {
            "Name": "subnet1",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1b",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet1"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 67,
          "width": null,
          "height": null,
        },
      },
    },
  };

  const specs = [];

  const vpcSpec = {
    "kind": "VPC",
    "properties": {
      "si": {
        "name": "vpc",
        "type": "configurationFrameDown",
        "color": "#ff9900",
      },
      "domain": {
        "CidrBlock": "10.0.0.0/24",
        "EnableDnsHostnames": true,
        "EnableDnsResolution": false,
        "awsResourceType": "vpc",
        "tags": {},
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 500.0,
      "width": 489.0,
      "height": 660.0,
    },
    "connect": null,
    "parent": null,
  };
  specs.push(vpcSpec);

  const subnet1Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet1",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1b",
        "IsPublic": true,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 555.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet1Spec);

  const subnet2Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet2",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1a",
        "IsPublic": false,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 56.0,
      "y": 804.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet2Spec);

  const expectedUpdates = {
    "status": "ok",
    "message": "Updated Components",
    "ops": {},
  };
  const result = converge("tt", thisComponent, components, specs as any);
  assertObjectMatch(result, expectedUpdates);
});

Deno.test(function convergeHandlesRevertsManualUpdates() {
  const thisComponent = {
    "kind": "TemplateTest",
    "properties": {
      "si": {
        "name": "template test",
        "type": "component",
        "color": "#00ffff",
      },
      "resource_value": {},
    },
    "geometry": {
      "tt": {
        "x": -18,
        "y": -488,
        "width": 0,
        "height": 0,
      },
    },
  };

  const components = {
    "01JJT8A42J83RHBQ44N6T75YFR": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet2",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1c",
          "IsPublic": false,
          "Tags": {
            "Name": "subnet2",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1a",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet2"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 38,
          "y": 316,
          "width": null,
          "height": null,
        },
      },
    },
    "01JJT8A41VFT17AJMTSWE50BT2": {
      "kind": "VPC",
      "properties": {
        "si": {
          "name": "vpc",
          "type": "configurationFrameDown",
          "color": "#ff9900",
        },
        "domain": {
          "CidrBlock": "10.0.0.0/24",
          "EnableDnsHostnames": true,
          "EnableDnsResolution": false,
          "awsResourceType": "vpc",
          "tags": {
            "Name": "vpc",
          },
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsVpcJSON": {
            "code":
              '{\n\t"CidrBlock": "10.0.0.0/24",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "vpc",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "vpc"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsVpcCanCreate": {
            "result": "failure",
            "message": "no Region available",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 12,
          "width": 489,
          "height": 660,
        },
      },
    },
    "01JJT8A428SCERDG35B106PB77": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet1",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1b",
          "IsPublic": true,
          "Tags": {
            "Name": "subnet1",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1b",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet1"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 67,
          "width": null,
          "height": null,
        },
      },
    },
  };

  const specs = [];

  const vpcSpec = {
    "kind": "VPC",
    "properties": {
      "si": {
        "name": "vpc",
        "type": "configurationFrameDown",
        "color": "#ff9900",
      },
      "domain": {
        "CidrBlock": "10.0.0.0/24",
        "EnableDnsHostnames": true,
        "EnableDnsResolution": false,
        "awsResourceType": "vpc",
        "tags": {},
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 500.0,
      "width": 489.0,
      "height": 660.0,
    },
    "connect": null,
    "parent": null,
  };
  specs.push(vpcSpec);

  const subnet1Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet1",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1b",
        "IsPublic": true,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 555.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet1Spec);

  const subnet2Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet2",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1a",
        "IsPublic": false,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 56.0,
      "y": 804.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet2Spec);

  const expectedUpdates = {
    "status": "ok",
    "message": "Updated Components",
    "ops": {},
  };
  const result = converge("tt", thisComponent, components, specs as any);
  assertObjectMatch(result, expectedUpdates);
});

Deno.test(function convergeHandlesDelete() {
  const thisComponent = {
    "kind": "TemplateTest",
    "properties": {
      "si": {
        "name": "template test",
        "type": "component",
        "color": "#00ffff",
      },
      "resource_value": {},
    },
    "geometry": {
      "tt": {
        "x": -18,
        "y": -488,
        "width": 0,
        "height": 0,
      },
    },
  };

  const components = {
    "01JJT8A42J83RHBQ44N6T75YFR": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet2",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1c",
          "IsPublic": false,
          "Tags": {
            "Name": "subnet2",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1a",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet2"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 38,
          "y": 316,
          "width": null,
          "height": null,
        },
      },
    },
    "01JJT8A41VFT17AJMTSWE50BT2": {
      "kind": "VPC",
      "properties": {
        "si": {
          "name": "vpc",
          "type": "configurationFrameDown",
          "color": "#ff9900",
        },
        "domain": {
          "CidrBlock": "10.0.0.0/24",
          "EnableDnsHostnames": true,
          "EnableDnsResolution": false,
          "awsResourceType": "vpc",
          "tags": {
            "Name": "vpc",
          },
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsVpcJSON": {
            "code":
              '{\n\t"CidrBlock": "10.0.0.0/24",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "vpc",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "vpc"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsVpcCanCreate": {
            "result": "failure",
            "message": "no Region available",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 12,
          "width": 489,
          "height": 660,
        },
      },
    },
    "01JJT8A428SCERDG35B106PB77": {
      "kind": "Subnet",
      "properties": {
        "si": {
          "name": "subnet1",
          "type": "component",
          "color": "#ff9900",
        },
        "domain": {
          "AvailabilityZone": "us-east-1b",
          "IsPublic": true,
          "Tags": {
            "Name": "subnet1",
          },
          "awsResourceType": "subnet",
        },
        "secrets": {},
        "resource_value": {},
        "code": {
          "si:generateAwsSubnetJSON": {
            "code":
              '{\n\t"AvailabilityZone": "us-east-1b",\n\t"TagSpecifications": [\n\t\t{\n\t\t\t"ResourceType": "subnet",\n\t\t\t"Tags": [\n\t\t\t\t{\n\t\t\t\t\t"Key": "Name",\n\t\t\t\t\t"Value": "subnet1"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}',
            "format": "json",
          },
        },
        "qualification": {
          "si:qualificationAwsSubnetCanCreate": {
            "result": "failure",
            "message": "no CidrBlock supplied",
          },
        },
      },
      "geometry": {
        "tt": {
          "x": 32,
          "y": 67,
          "width": null,
          "height": null,
        },
      },
    },
  };

  const specs = [];

  const vpcSpec = {
    "kind": "VPC",
    "properties": {
      "si": {
        "name": "vpc",
        "type": "configurationFrameDown",
        "color": "#ff9900",
      },
      "domain": {
        "CidrBlock": "10.0.0.0/24",
        "EnableDnsHostnames": true,
        "EnableDnsResolution": false,
        "awsResourceType": "vpc",
        "tags": {},
      },
    },
    "geometry": {
      "x": 50.0,
      "y": 500.0,
      "width": 489.0,
      "height": 660.0,
    },
    "connect": null,
    "parent": null,
  };
  specs.push(vpcSpec);

  const subnet2Spec = {
    "kind": "Subnet",
    "properties": {
      "si": {
        "name": "subnet2",
        "type": "component",
        "color": "#ff9900",
      },
      "domain": {
        "AvailabilityZone": "us-east-1a",
        "IsPublic": false,
        "Tags": {},
        "awsResourceType": "subnet",
      },
    },
    "geometry": {
      "x": 56.0,
      "y": 804.0,
      "width": null,
      "height": null,
    },
    "connect": null,
    "parent": "vpc",
  };
  specs.push(subnet2Spec);

  const expectedUpdates = {
    "status": "ok",
    "message": "Updated Components",
    "ops": {
      "delete": ["subnet1"],
    },
  };
  const result = converge("tt", thisComponent, components, specs as any);
  assertObjectMatch(result, expectedUpdates);
});
