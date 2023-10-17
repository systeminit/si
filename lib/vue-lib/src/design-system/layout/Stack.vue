/* General "stack" component, which helps arrange a vertical stack of components
with even spacing between them */

<script lang="ts">
import { h, VNode, PropType, Fragment } from "vue";
import { responsiveSpacingProps } from "../utils/size_utils";
import {
  FunctionalComponentContextArg,
  FunctionalComponentPropsType,
  getSlotChildren,
} from "../../utils/vue_utils";
import Divider from "./Divider.vue";

const propsDefinition = {
  ...responsiveSpacingProps("sm"),
  tagName: { type: String as PropType<"div" | "section">, default: "div" },
  dividers: Boolean,
} as const;

const Stack = (
  props: FunctionalComponentPropsType<typeof propsDefinition>,
  context: FunctionalComponentContextArg,
): VNode => {
  const classes = {
    stack: true,
    [`--spacing-${props.spacing}`]: true,
    [`--spacing-${props.spacingMobile}-mobile-only`]: !!props.spacingMobile,
    [`--spacing-${props.spacingTablet}-tablet`]: !!props.spacingTablet,
    [`--spacing-${props.spacingDesktop}-desktop`]: !!props.spacingDesktop,
    [`--spacing-${props.spacingWide}-wide`]: !!props.spacingWide,
    [`--spacing-${props.spacingHuge}-huge`]: !!props.spacingHuge,
  };

  const wrappedChildren = [] as VNode[];
  const children = getSlotChildren(context.slots.default);
  for (let i = 0; i < children.length; i++) {
    if (!children[i]) continue;

    // NOTE - ran into a weird errors that only appeared on the built version of the app
    // but resolved it by adding this Fragment wrapper around each child
    wrappedChildren.push(h(Fragment, {}, [children[i]]));
    if (props.dividers && i < children.length - 1) {
      wrappedChildren.push(h(Divider));
    }
  }

  return h(
    props.tagName || "div",
    {
      class: classes,
    },
    wrappedChildren,
  );
};

Stack.props = propsDefinition;

export default Stack;
</script>

<style lang="less" scoped>
.stack {
  display: flex;
  flex-direction: column;
  row-gap: @spacing-rem[xs];

  // note - we use spacing-rem so sizes adjust with base font size
  each(@spacing-rem, .(@size-px, @size-name){
    &.--spacing-@{size-name} {
      row-gap: @size-px;
    }
  })

  each(@breakpoints, .(@bp-name){
    @mq-var-name: ~'mq-@{bp-name}';
    @media @@mq-var-name {
      each(@spacing-rem, .(@size-px, @size-name){
        &.--spacing-@{size-name}-@{bp-name} {
          row-gap: @size-px;
        }
      })
    }
  });
}

// TODO: add hacky good enough fix for no flex gap support :(
// see https://ppuzio.medium.com/flexbox-gap-workaround-for-safari-on-ios-14-13-and-lower-ffcae589eb69

// remove margins on paragraphs, we may do this globally instead so we can remove it?
:deep(.stack) {
  > p {
    margin: 0;
  }
}
</style>
