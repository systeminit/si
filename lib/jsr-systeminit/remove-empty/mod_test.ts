import { assertEquals } from "@std/assert";
import { removeEmpty } from "./mod.ts";

Deno.test(function simpleObject() {
  const startingObj = {
    "one": "one",
    "two": {},
    "three": undefined,
    "four": null,
    "five": [],
  };
  const finishedObj = {
    "one": "one",
  };
  assertEquals(removeEmpty(startingObj), finishedObj);
});

Deno.test(function nestedObject() {
  const startingObj = {
    "one": "one",
    "two": {},
    "three": undefined,
    "four": null,
    "five": [],
    "six": {
      "seven": {},
    },
  };
  const finishedObj = {
    "one": "one",
  };
  assertEquals(removeEmpty(startingObj), finishedObj);
});

Deno.test(function nestedArray() {
  const startingObj = {
    "one": "one",
    "two": {},
    "three": undefined,
    "four": null,
    "five": [],
    "six": {
      "seven": {
        "eight": [],
      },
    },
  };
  const finishedObj = {
    "one": "one",
  };
  assertEquals(removeEmpty(startingObj), finishedObj);
});

Deno.test(function nestedArrayWithEmptyObject() {
  const startingObj = {
    "one": "one",
    "two": {},
    "three": undefined,
    "four": null,
    "five": [],
    "six": {
      "seven": {
        "eight": [{}],
      },
    },
  };
  const finishedObj = {
    "one": "one",
  };
  assertEquals(removeEmpty(startingObj), finishedObj);
});

Deno.test(function nestedArrayWithEmptyObjectNulls() {
  const startingObj = {
    "one": "one",
    "two": {},
    "three": undefined,
    "four": null,
    "five": [],
    "six": {
      "seven": {
        "eight": [{}, null, undefined, {}],
      },
    },
  };
  const finishedObj = {
    "one": "one",
  };
  assertEquals(removeEmpty(startingObj), finishedObj);
});

Deno.test(function autoScaleGroup() {
  const startingObj = {
    "TypeName": "AWS::AutoScaling::AutoScalingGroup",
    "DesiredState": {
      "MaxSize": "3",
      "MinSize": "3",
      "CapacityReservationSpecification": {
        "CapacityReservationTarget": {},
      },
      "LaunchTemplate": {
        "LaunchTemplateName": "funky",
        "Version": "1",
      },
      "MixedInstancesPolicy": {
        "LaunchTemplate": {},
      },
      "VPCZoneIdentifier": [
        "subnet-0abeed19b31871e40",
        "subnet-024c8c85874c47841",
        "subnet-0ce0acf3ada90fbf5",
      ],
    },
  };
  const finishObj = {
    "TypeName": "AWS::AutoScaling::AutoScalingGroup",
    "DesiredState": {
      "MaxSize": "3",
      "MinSize": "3",
      "LaunchTemplate": {
        "LaunchTemplateName": "funky",
        "Version": "1",
      },
      "VPCZoneIdentifier": [
        "subnet-0abeed19b31871e40",
        "subnet-024c8c85874c47841",
        "subnet-0ce0acf3ada90fbf5",
      ],
    },
  };

  assertEquals(removeEmpty(startingObj), finishObj);
});
