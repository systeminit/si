import { Slot, VNode, Comment, Fragment, DefineComponent } from "vue";

// helper for wrapping slot children in functional components
export function getSlotChildren(slot: Slot | undefined): VNode[] {
  return unwrapChildren(slot?.() || []);
}
function unwrapChildren(rawChildren: VNode[]) {
  const children = [] as VNode[];
  rawChildren.forEach((child) => {
    // components that are hidden via v-if end up as a Comment <!--v-if--> so we skip over them
    if (child.type === Comment) return;

    // if the child is a template with a v-for, we actually want to get all of its children
    if (child.type === Fragment) {
      children.push(...unwrapChildren(child.children as VNode[]));
    } else {
      children.push(child);
    }
  });
  return children;
}

// functional component TS helper
// use by setting props arg type
// example:
// `const FunctionalComponent = (props: FunctionalComponentPropsType<typeof propsDefinition>, ctx: any) => ...`
// see https://github.com/johnsoncodehk/volar/issues/924
export type FunctionalComponentPropsType<T> = InstanceType<
  DefineComponent<T>
>["$props"];

// TODO: get the right type - see same issue above...
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type FunctionalComponentContextArg = any;
