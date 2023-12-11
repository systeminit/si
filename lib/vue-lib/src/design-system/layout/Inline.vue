/* General "inline" component, which helps arrange a horizontal row of
components with even spacing between them, wrapping to the next line if
necessary NOTE - also converted to functional component to make HMR work and
improve perf */
<script lang="ts">
import { h, VNode, PropType } from "vue";
import {
  BreakpointNamesWithoutSmallest,
  HorizontalAlignmentValues,
  responsiveAlignProps,
  responsiveSpacingProps,
  responsiveVerticalAlignProps,
} from "../utils/size_utils";
import {
  FunctionalComponentContextArg,
  FunctionalComponentPropsType,
  getSlotChildren,
} from "../../utils/vue_utils";

const propsDefinition = {
  ...responsiveSpacingProps("xs"),
  ...responsiveAlignProps,
  ...responsiveVerticalAlignProps,
  collapseBelow: { type: String as PropType<BreakpointNamesWithoutSmallest> },
  tagName: {
    type: String as PropType<"div" | "section" | "ol" | "ul" | "li">,
    default: "div",
  },
  reverse: {
    type: Boolean,
    default: false,
  },
  noWrap: Boolean,
  fullWidth: Boolean,
} as const;

function flipAlignIfReversed(
  reverse: boolean | undefined,
  alignValue?: HorizontalAlignmentValues,
) {
  if (!reverse) return alignValue;
  if (alignValue === "left") return "right";
  if (alignValue === "right") return "left";
  return alignValue;
}

const Inline = (
  props: FunctionalComponentPropsType<typeof propsDefinition>,
  context: FunctionalComponentContextArg,
): VNode => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const isList = ["ol", "ul"].includes(props.tagName!);

  const classes = {
    "ui-inline": true,
    [`--spacing-${props.spacing}`]: true,
    [`--spacing-${props.spacingMobile}-mobile-only`]: !!props.spacingMobile,
    [`--spacing-${props.spacingTablet}-tablet`]: !!props.spacingTablet,
    [`--spacing-${props.spacingDesktop}-desktop`]: !!props.spacingDesktop,
    [`--spacing-${props.spacingWide}-wide`]: !!props.spacingWide,
    [`--spacing-${props.spacingHuge}-huge`]: !!props.spacingHuge,

    // need to flip the alignment option if reversed option is being used
    [`--align-${flipAlignIfReversed(props.reverse, props.align)}`]: true,
    [`--align-${flipAlignIfReversed(
      props.reverse,
      props.alignMobile,
    )}-mobile-only`]: !!props.alignMobile,
    [`--align-${flipAlignIfReversed(props.reverse, props.alignTablet)}-tablet`]:
      !!props.alignTablet,
    [`--align-${flipAlignIfReversed(
      props.reverse,
      props.alignDesktop,
    )}-desktop`]: !!props.alignDesktop,
    [`--align-${flipAlignIfReversed(props.reverse, props.alignWide)}-wide`]:
      !!props.alignWide,
    [`--align-${flipAlignIfReversed(props.reverse, props.alignHuge)}-huge`]:
      !!props.alignHuge,

    [`--aligny-${props.alignY}`]: true,
    [`--aligny-${props.alignYMobile}-mobile-only`]: !!props.alignYMobile,
    [`--aligny-${props.alignYTablet}-tablet`]: !!props.alignYTablet,
    [`--aligny-${props.alignYDesktop}-desktop`]: !!props.alignYDesktop,
    [`--aligny-${props.alignYWide}-wide`]: !!props.alignYWide,
    [`--aligny-${props.alignYHuge}-huge`]: !!props.alignYHuge,

    "--reverse": props.reverse,

    ...(props.collapseBelow && {
      [`--collapse-below-${props.collapseBelow}`]: true,
    }),

    "--no-wrap": props.noWrap,
    "--full-width": props.fullWidth,
  };

  const wrappedChildren = [] as VNode[];
  const children = getSlotChildren(context.slots.default);
  for (let i = 0; i < children.length; i++) {
    const item = h(
      isList ? "li" : "div",
      { class: "inline__item" },
      children[i],
    );
    // item.children = [];
    wrappedChildren.push(item);
  }

  return h(
    props.tagName || "div",
    {
      class: classes,
    },
    wrappedChildren,
  );
};

Inline.props = propsDefinition;

export default Inline;
</script>

<style lang="less">
// avoiding the class name "inline" to not conflict with tailwind
.ui-inline {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: @spacing-rem[xs];

  // could set this up to only be there if tagname is ol/ul, but doesn't hurt anything
  list-style: none;
  margin: 0;
  padding: 0;

  &.--full-width {
    > .inline__item {
      flex-grow: 1;
      display: flex;
      flex-direction: column;
      > .vbutton {
        flex-grow: 1;
      }
    }
  }

  &.--reverse {
    flex-direction: row-reverse;
  }

  &.--no-wrap {
    flex-wrap: nowrap;
  }

  each(@spacing-rem, .(@size-px, @size-name){
    &.--spacing-@{size-name} {
      gap: @size-px;
    }
  })
  each(@horizontal-align-options, .(@flex-value, @option-name){
    &.--align-@{option-name} {
      justify-content: @flex-value;
    }
  })
  each(@vertical-align-options, .(@flex-value, @option-name){
    &.--aligny-@{option-name} {
      align-items: @flex-value;
    }
  })

  each(@breakpoints, .(@bp-name){
    @mq-var-name: ~'mq-@{bp-name}';
    @media @@mq-var-name {
      each(@spacing-rem, .(@size-px, @size-name){
        &.--spacing-@{size-name}-@{bp-name} {
          gap: @size-px;
        }
      })
      each(@horizontal-align-options, .(@flex-value, @option-name){
        &.--align-@{option-name}-@{bp-name} {
          justify-content: @flex-value;
        }
      })
      each(@vertical-align-options, .(@flex-value, @option-name){
        &.--aligny-@{option-name}-@{bp-name} {
          align-items: @flex-value;
        }
      })
    }
  })

  each(@breakpoints-without-smallest, .(@bp-name){
    @mq-var-name: ~'mq-below-@{bp-name}';
    @media @@mq-var-name {
      &.--collapse-below-@{bp-name} {
        flex-direction: column;
        &.--reverse {
          flex-direction: column-reverse;
        }
      }
    }
  });

  > .inline__item {
    &:empty {
      display: none;
    }
  }
}

// TODO: add hacky good enough fix for no flex gap support :(
// see https://ppuzio.medium.com/flexbox-gap-workaround-for-safari-on-ios-14-13-and-lower-ffcae589eb69
</style>
