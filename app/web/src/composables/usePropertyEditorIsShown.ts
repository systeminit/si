import { Ref, computed } from "vue";
import _ from "lodash";
import { PropertyPath } from "@/api/sdf/dal/property_editor";

export function usePropertyEditorIsShown(
  _name: Ref<string>,
  collapsedPaths: Ref<Array<Array<string>>>,
  path?: Ref<PropertyPath | undefined>,
  isHeader?: boolean,
) {
  const isShown = computed(() => {
    if (path && path.value) {
      const checkPath = _.cloneDeep(path.value.triggerPath);
      checkPath.reverse();
      for (const c of collapsedPaths.value) {
        const reverseCollapsedPath = _.cloneDeep(c);
        reverseCollapsedPath.reverse();
        if (checkPath.length >= reverseCollapsedPath.length) {
          const checkPathSlice = checkPath.slice(
            0,
            reverseCollapsedPath.length,
          );
          if (_.isEqual(checkPathSlice, reverseCollapsedPath)) {
            if (isHeader) {
              if (!_.isEqual(checkPath, reverseCollapsedPath)) {
                return false;
              }
            } else {
              return false;
            }
          }
        }
      }
    }
    return true;
  });

  const isCollapsed = computed(() => {
    if (path && path.value) {
      const checkPath = _.cloneDeep(path.value.triggerPath);
      checkPath.reverse();
      for (const c of collapsedPaths.value) {
        const reverseCollapsedPath = _.cloneDeep(c);
        reverseCollapsedPath.reverse();
        if (_.isEqual(checkPath, reverseCollapsedPath)) {
          return true;
        }
      }
    }
    return false;
  });

  return { isShown, isCollapsed };
}
