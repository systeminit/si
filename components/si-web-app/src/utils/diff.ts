import { Entity } from "@/store/modules/entity";
import _ from "lodash";

export interface DiffEntry {
  path: (string | number)[];
  before: any | undefined;
  after: any;
  kind: string;
}

export interface DiffResult {
  entries: DiffEntry[];
  count: number;
}

export function diffEntity(lhs: Entity | null, rhs: Entity): DiffResult {
  const result: DiffResult = {
    entries: [],
    count: 0,
  };

  // Is the lhs null? if so, we've got no changes.
  if (!lhs) {
    return result;
  }

  let checkPaths: DiffEntry["path"][] = _.map(
    ["name", "displayName", "description", "properties"],
    key => [key],
  );

  for (const path of checkPaths) {
    let rhsValue = _.get(rhs, path);
    const lhsValue = _.get(lhs, path);

    // If the two sides are equal, then there is no diff - so
    // skip this path!
    if (_.isEqual(lhsValue, rhsValue)) {
      continue;
    }

    if (_.isArray(rhsValue)) {
      // If the value looks like a map, we have to do some fancy
      // footwork. For each entry in the map, check and see if there
      // is a key/value pair that matches.
      if (
        rhsValue[0] &&
        rhsValue[0].hasOwnProperty("key") &&
        rhsValue[0].hasOwnProperty("value")
      ) {
        let sortedValue = _.sortBy(rhsValue, ["key"]);
        rhsValue = sortedValue;
        //_.set(rhs, path, sortedValue);
        for (let x = 0; x < rhsValue.length; x++) {
          let rhsEntry = rhsValue[x];
          let matchEntry = _.find(lhsValue, lhsEntry => {
            return lhsEntry.key == rhsEntry.key;
          });
          if (matchEntry) {
            if (_.isEqual(matchEntry.value, rhsEntry.value)) {
              continue;
            }
            const newPath = _.cloneDeep(path);
            newPath.push(x);
            newPath.push("value");
            result.entries.push({
              path: newPath,
              before: matchEntry.value,
              after: rhsEntry.value,
              kind: "edit",
            });
          } else {
            const keyPath = _.cloneDeep(path);
            keyPath.push(x);
            keyPath.push("key");
            result.entries.push({
              path: keyPath,
              before: undefined,
              after: rhsEntry.key,
              kind: "add",
            });
            const valuePath = _.cloneDeep(path);
            valuePath.push(x);
            valuePath.push("value");
            result.entries.push({
              path: valuePath,
              before: undefined,
              after: rhsEntry.value,
              kind: "add",
            });
          }
        }
        continue;
      }
      // Add every entry of this object as a path to check
      for (let x = 0; x < rhsValue.length; x++) {
        const newPath = _.cloneDeep(path);
        newPath.push(x);
        checkPaths.push(newPath);
      }
      if (lhsValue && lhsValue.length > rhsValue.length) {
        for (let x = rhsValue.length; x < lhsValue.length; x++) {
          const newPath = _.cloneDeep(path);
          newPath.push(x);
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
        kind,
      });
    }
  }

  result.count = result.entries.length;
  return result;
}
