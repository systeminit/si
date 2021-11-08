import { Entity } from "@/api/sdf/model/entity";
import _ from "lodash";

export interface DiffEntry {
  path: string[];
  before: any | undefined;
  after: any;
  kind: "edit" | "add" | "delete";
}

export interface DiffResult {
  entries: DiffEntry[];
  count: number;
}

export function diffEntity(lhs: Entity, rhs: Entity): DiffResult {
  const result: DiffResult = {
    entries: [],
    count: 0,
  };

  let checkPaths: string[][] = [["name"], ["properties", "__baseline"]];

  for (const path of checkPaths) {
    let rhsValue = _.get(rhs, path);
    const lhsValue = _.get(lhs, path);

    // If the two sides are equal, then there is no diff - so
    // skip this path!
    if (_.isEqual(lhsValue, rhsValue)) {
      continue;
    }

    if (_.isArray(rhsValue)) {
      // Add every entry of this object as a path to check
      for (let x = 0; x < rhsValue.length; x++) {
        const newPath = _.cloneDeep(path);
        newPath.push(`${x}`);
        checkPaths.push(newPath);
      }
      if (lhsValue && lhsValue.length > rhsValue.length) {
        for (let x = rhsValue.length; x < lhsValue.length; x++) {
          const newPath = _.cloneDeep(path);
          newPath.push(`${x}`);
          checkPaths.push(newPath);
        }
      }
    } else if (_.isObjectLike(rhsValue)) {
      // Add every key of this object as a path to check
      for (const key in rhsValue) {
        const newPath = _.cloneDeep(path);
        newPath.push(key);
        checkPaths.push(newPath);
      }
    } else {
      let kind = "edit";
      if (lhsValue == undefined) {
        kind = "add";
      }
      if (rhsValue == undefined) {
        kind = "delete";
      }
      result.entries.push({
        path: path,
        before: lhsValue,
        after: rhsValue,
        // @ts-ignore
        kind,
      });
    }

    // Check for deleted fields - they would present as
    // fields present in the lhs, but missing in the rhs.
    if (_.isObjectLike(lhsValue)) {
      for (const key in lhsValue) {
        if (!_.get(rhsValue, key)) {
          const newPath = _.cloneDeep(path);
          newPath.push(key);
          checkPaths.push(newPath);
        }
      }
    }
  }

  result.count = result.entries.length;
  return result;
}
