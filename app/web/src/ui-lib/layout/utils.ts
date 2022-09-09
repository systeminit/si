import { Comment, DefineComponent, Fragment, PropType, Slot, VNode } from "vue";
import _ from "lodash";

export type BreakpointNames = "mobile" | "tablet" | "desktop" | "wide" | "huge";
export type BreakpointNamesWithoutMobile = Exclude<BreakpointNames, "mobile">;
export type BreakpointNamesWithoutLargest = Exclude<BreakpointNames, "huge">;

export type SpacingSizes =
  | "none"
  | "xxs"
  | "xs"
  | "s"
  | "m"
  | "l"
  | "xl"
  | "xxl";

// TODO: maybe there is a way to get these objects generated with a helper rather than manually creating them?
// I tried with no luck... Not a huge deal to define manually

export const responsiveSpacingProps = (defaultSpacing: SpacingSizes) => ({
  spacing: { type: String as PropType<SpacingSizes>, default: defaultSpacing },
  spacingMobile: { type: String as PropType<SpacingSizes> },
  spacingTablet: { type: String as PropType<SpacingSizes> },
  spacingDesktop: { type: String as PropType<SpacingSizes> },
  spacingWide: { type: String as PropType<SpacingSizes> },
  spacingHuge: { type: String as PropType<SpacingSizes> },
});

export const responsivePaddingProps = (defaultPadding?: SpacingSizes) => ({
  padding: { type: String as PropType<SpacingSizes>, default: defaultPadding },
  paddingMobile: { type: String as PropType<SpacingSizes> },
  paddingTablet: { type: String as PropType<SpacingSizes> },
  paddingDesktop: { type: String as PropType<SpacingSizes> },
  paddingWide: { type: String as PropType<SpacingSizes> },
  paddingHuge: { type: String as PropType<SpacingSizes> },
});

export type HorizontalAlignmentValues = "left" | "center" | "right";
export const responsiveAlignProps = {
  align: {
    type: String as PropType<HorizontalAlignmentValues>,
    default: "left",
  },
  alignMobile: { type: String as PropType<HorizontalAlignmentValues> },
  alignTablet: { type: String as PropType<HorizontalAlignmentValues> },
  alignDesktop: { type: String as PropType<HorizontalAlignmentValues> },
  alignWide: { type: String as PropType<HorizontalAlignmentValues> },
  alignHuge: { type: String as PropType<HorizontalAlignmentValues> },
};

export type VerticalAlignmentValues = "top" | "center" | "bottom";
export const responsiveVerticalAlignProps = {
  alignY: { type: String as PropType<VerticalAlignmentValues>, default: "top" },
  alignYMobile: { type: String as PropType<VerticalAlignmentValues> },
  alignYTablet: { type: String as PropType<VerticalAlignmentValues> },
  alignYDesktop: { type: String as PropType<VerticalAlignmentValues> },
  alignYWide: { type: String as PropType<VerticalAlignmentValues> },
  alignYHuge: { type: String as PropType<VerticalAlignmentValues> },
};

// note - slightly easier to use strings so components dont need to bind with `columns="2"` rather than `:columns="2"`
export type NumColumnsValues = "1" | "2" | "3" | "4" | "5" | "6";
export const responsiveNumColumnsProps = {
  columns: { type: String as PropType<NumColumnsValues>, default: "3" },
  columnsMobile: { type: String as PropType<NumColumnsValues> },
  columnsTablet: { type: String as PropType<NumColumnsValues> },
  columnsDesktop: { type: String as PropType<NumColumnsValues> },
  columnsWide: { type: String as PropType<NumColumnsValues> },
  columnsHuge: { type: String as PropType<NumColumnsValues> },
};

export type HeadingTextSizes =
  | "xxs"
  | "xs"
  | "s"
  | "m"
  | "m2"
  | "l"
  | "xl"
  | "xxl";
export const responsiveTextSizeProps = {
  size: { type: String as PropType<HeadingTextSizes>, default: "m" },
  sizeMobile: { type: String as PropType<HeadingTextSizes> },
  sizeTablet: { type: String as PropType<HeadingTextSizes> },
  sizeDesktop: { type: String as PropType<HeadingTextSizes> },
  sizeWide: { type: String as PropType<HeadingTextSizes> },
  sizeHuge: { type: String as PropType<HeadingTextSizes> },
};

export type ColumnLayoutTemplate = string;
export const responsiveColumnLayoutProps = {
  layout: { type: String as PropType<ColumnLayoutTemplate> },
  layoutMobile: { type: String as PropType<ColumnLayoutTemplate> },
  layoutTablet: { type: String as PropType<ColumnLayoutTemplate> },
  layoutDesktop: { type: String as PropType<ColumnLayoutTemplate> },
  layoutWide: { type: String as PropType<ColumnLayoutTemplate> },
  layoutHuge: { type: String as PropType<ColumnLayoutTemplate> },
};

// helper for wrapping slot children in functional components
export function getSlotChildren(slot: Slot | undefined): VNode[] {
  const rawChildren = slot?.() || [];
  const children = [] as VNode[];
  rawChildren.forEach((child) => {
    // components that are hidden via v-if end up as a Comment <!--v-if--> so we skip over them
    if (child.type === Comment) return;

    // if the child is a template with a v-for, we actually want to get all of its children
    if (child.type === Fragment) {
      children.push(...(child.children as VNode[]));
    } else {
      children.push(child);
    }
  });
  return children as VNode[];
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
