import { Ref, computed } from "vue";
import _ from "lodash";

export function usePropertyEditorIsShown(
  name: Ref<string>,
  path: Ref<string[]>,
  collapsedPaths: Ref<Array<Array<string>>>,
) {
  const isShown = computed(() => {
    const checkPath = [name.value, ...path.value].reverse();
    for (const c of collapsedPaths.value) {
      const reverseCollapsedPath = _.cloneDeep(c);
      reverseCollapsedPath.reverse();
      if (checkPath.length >= reverseCollapsedPath.length) {
        const checkPathSlice = checkPath.slice(0, reverseCollapsedPath.length);
        if (_.isEqual(checkPathSlice, reverseCollapsedPath)) {
          if (!_.isEqual(checkPath, reverseCollapsedPath)) {
            return false;
          }
        }
      }
    }
    return true;
  });

  const isCollapsed = computed(() => {
    const checkPath = [name.value, ...path.value].reverse();
    for (const c of collapsedPaths.value) {
      const reverseCollapsedPath = _.cloneDeep(c);
      reverseCollapsedPath.reverse();
      if (_.isEqual(checkPath, reverseCollapsedPath)) {
        return true;
      }
    }
    return false;
  });

  return { isShown, isCollapsed };
}
