/**
 * Removes empty elements from a target object. Useful any time you want to
 * render JSON or YAML, but there might be empty objects, arrays, null, or
 * undefined that would break the schema.
 *
 * For example, it transforms:
 *
 * ```json
 *  const startingObj = {
 *    "TypeName": "AWS::AutoScaling::AutoScalingGroup",
 *    "DesiredState": {
 *      "MaxSize": "3",
 *      "MinSize": "3",
 *      "CapacityReservationSpecification": {
 *        "CapacityReservationTarget": {},
 *      },
 *      "LaunchTemplate": {
 *        "LaunchTemplateName": "funky",
 *        "Version": "1",
 *      },
 *      "MixedInstancesPolicy": {
 *        "LaunchTemplate": {},
 *      },
 *      "VPCZoneIdentifier": [
 *        "subnet-0abeed19b31871e40",
 *        "subnet-024c8c85874c47841",
 *        "subnet-0ce0acf3ada90fbf5",
 *      ],
 *    },
 *  };
 * ```
 *
 * To this:
 *
 * ```json
 *  const finishObj = { "TypeName": "AWS::AutoScaling::AutoScalingGroup",
 *    "DesiredState": {
 *      "MaxSize": "3",
 *      "MinSize": "3",
 *      "LaunchTemplate": {
 *        "LaunchTemplateName": "funky",
 *        "Version": "1",
 *      },
 *      "VPCZoneIdentifier": [
 *        "subnet-0abeed19b31871e40",
 *        "subnet-024c8c85874c47841",
 *        "subnet-0ce0acf3ada90fbf5",
 *      ],
 *    },
 *  };
 * ```
 * It will remove any empty objects, arrays, null, or undefined values
 *
 * To use it:
 *
 * @example
 * ```ts
 * import { removeEmpty } from "jsr:@systeminit/remove-empty@0";
 *
 * const obj = { foo: {} };
 * removeEmpty(obj);
 * ```
 * @module
 */

import _ from "npm:lodash@4";

/**
 * Removes empty objects, arrays, null, or undefined values from a target
 * object, regardless of how deeply nested they may be.
 *
 * @param targetObject the target object to remove empty values from.
 */
export function removeEmpty(targetObject: any): any {
  const stack: { path: Array<string> }[] = [{ path: [] }];
  for (const key of Object.keys(targetObject)) {
    stack.push({ path: [key] });
  }

  while (stack.length) {
    const next = stack.pop();
    let path: Array<string>;
    if (next) {
      path = next.path;
    } else {
      throw new Error("stack length was true, but no object. Bug!");
    }
    let value = _.get(targetObject, path);
    if (_.isObject(value)) {
      // This checks for sparse arrays, and filters out their entries.
      // This happens if we unset a deep value
      if (_.isArray(value)) {
        const newValues = Object.values(value);
        _.set(targetObject, path, newValues);
        value = newValues;
      }
      if (_.isEmpty(value)) {
        _.unset(targetObject, path);
        if (path.length > 1) {
          const newPath = _.cloneDeep(path);
          newPath.pop();
          stack.push({ path: newPath });
        }
      } else {
        for (const key of Object.keys(value)) {
          const newPath = _.cloneDeep(path);
          newPath.push(key);
          stack.push({ path: newPath });
        }
      }
    } else if (_.isUndefined(value)) {
      _.unset(targetObject, path);
    } else if (_.isNull(value)) {
      _.unset(targetObject, path);
    }
  }
  return targetObject;
}
