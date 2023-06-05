import { PropType } from "vue";

export type BreakpointNames = "mobile" | "tablet" | "desktop" | "wide" | "huge";
export type BreakpointNamesWithoutSmallest = Exclude<BreakpointNames, "mobile">;
export type BreakpointNamesWithoutLargest = Exclude<BreakpointNames, "huge">;

export type SpacingSizes =
  | "none"
  | "2xs"
  | "xs"
  | "sm"
  | "md"
  | "lg"
  | "xl"
  | "2xl";

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

export type HeadingTextSizes = "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl";
export const responsiveTextSizeProps = {
  size: { type: String as PropType<HeadingTextSizes>, default: "md" },
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
