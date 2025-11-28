/**
 * Component filtering utilities for component-list tool.
 * This module is separate from componentList.ts to avoid importing si_client
 * dependencies during tests.
 */

export interface ComponentListItem {
  componentId: string;
  componentName: string;
  schemaName: string;
}

export interface FilterGroup {
  responseField: "componentName" | "componentId" | "schemaName";
  logic?: "AND" | "OR";
  regularExpressions: string[];
}

export interface Filters {
  logic?: "AND" | "OR";
  filterGroups: FilterGroup[];
}

/**
 * Applies filters to a list of components using regex patterns.
 * Supports AND/OR logic both within filter groups and between them.
 *
 * @param components - Array of components to filter
 * @param filters - Filter configuration with logic and filter groups
 * @returns Filtered array of components
 */
export function applyFilters(
  components: Array<ComponentListItem>,
  filters?: Filters,
): Array<ComponentListItem> {
  if (!filters || !filters.filterGroups || filters.filterGroups.length === 0) {
    return components;
  }

  const betweenGroupsLogic = filters.logic || "AND";

  return components.filter((component) => {
    const groupResults = filters.filterGroups.map((filterGroup) => {
      const fieldValue = component[filterGroup.responseField];
      const withinGroupLogic = filterGroup.logic || "OR";

      const regexResults = filterGroup.regularExpressions.map((regexStr) => {
        try {
          const regex = new RegExp(regexStr);
          return regex.test(fieldValue);
        } catch (error) {
          // If regex is invalid, skip this regex
          console.warn(`Invalid regex pattern: ${regexStr}`, error);
          return false;
        }
      });

      // Apply logic within the filter group
      if (withinGroupLogic === "AND") {
        return regexResults.every((result) => result);
      } else {
        return regexResults.some((result) => result);
      }
    });

    // Apply logic between filter groups
    if (betweenGroupsLogic === "AND") {
      return groupResults.every((result) => result);
    } else {
      return groupResults.some((result) => result);
    }
  });
}
