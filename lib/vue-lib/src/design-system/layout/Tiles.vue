/* Layout a grid of evenly spaced "tiles" */
<script lang="ts">
import { h, VNode } from "vue";
import {
  responsiveNumColumnsProps,
  responsiveSpacingProps,
} from "../utils/size_utils";
import {
  FunctionalComponentContextArg,
  FunctionalComponentPropsType,
  getSlotChildren,
} from "../../utils/vue_utils";
import type { PropType } from "vue";

const propsDefinition = {
  ...responsiveSpacingProps("md"),
  ...responsiveNumColumnsProps,
  // TODO: could add dividers option to show dividers when collapsed like braid's Tile component?
  tagName: { type: String as PropType<"div" | "section">, default: "div" },
} as const;

const Tiles = (
  props: FunctionalComponentPropsType<typeof propsDefinition>,
  context: FunctionalComponentContextArg,
): VNode => {
  const classes = {
    tiles: true,
    [`--spacing-${props.spacing}`]: true,
    [`--spacing-${props.spacingMobile}-mobile-only`]: !!props.spacingMobile,
    [`--spacing-${props.spacingTablet}-tablet`]: !!props.spacingTablet,
    [`--spacing-${props.spacingDesktop}-desktop`]: !!props.spacingDesktop,
    [`--spacing-${props.spacingWide}-wide`]: !!props.spacingWide,

    [`--columns-${props.columns}`]: true,
    [`--columns-${props.columnsMobile}-mobile-only`]: !!props.columnsMobile,
    [`--columns-${props.columnsTablet}-tablet`]: !!props.columnsTablet,
    [`--columns-${props.columnsDesktop}-desktop`]: !!props.columnsDesktop,
    [`--columns-${props.columnsWide}-wide`]: !!props.columnsWide,
  };

  const wrappedChildren = [] as VNode[];
  const children = getSlotChildren(context.slots.default);
  for (let i = 0; i < children.length; i++) {
    const item = h("div", { class: "tiles__item" }, children[i]);
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

Tiles.props = propsDefinition;

export default Tiles;
</script>

<style lang="less" scoped>
// move this elsewhere if we start using for any other components
@num-columns-options: range(6);

.getColumnsTemplate(@numColumns) {
  @range: range(@num);
  @templatestring: replace("@{range}", "\d", "1fr", "g");
  grid-template-columns: ~"@{templatestring}";
}
.tiles {
  display: grid;
  gap: @spacing-px[xs];

  each(@spacing-px, .(@size-px, @size-name){
    &.--spacing-@{size-name} {
      gap: @size-px;
    }
  })
  each(@num-columns-options, .(@num){
    &.--columns-@{num} {
      .getColumnsTemplate(@num);
    }
  })

  each(@breakpoints, .(@bp-name){
    @mq-var-name: ~'mq-@{bp-name}';
    @media @@mq-var-name {
      each(@spacing-px, .(@size-px, @size-name){
        &.--spacing-@{size-name}-@{bp-name} {
          gap: @size-px;
        }
      })
      each(@num-columns-options, .(@num){
        &.--columns-@{num}-@{bp-name} {
          .getColumnsTemplate(@num);
        }
      })

    }
  });
}

// TODO: add hacky good enough fix for no flex gap support :(
// see https://ppuzio.medium.com/flexbox-gap-workaround-for-safari-on-ios-14-13-and-lower-ffcae589eb69
</style>
