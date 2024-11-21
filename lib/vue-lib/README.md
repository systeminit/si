# @si/vue-lib

Place to keep any vue/frontend related tools that are going to be shared between other repos.

Note this is not for vue components / ui library, they (will) go in @si/ui-lib.

Put more general typescript tools that could be useful in a backend context in @si/ts-lib.

Some things may split into their own repos if they grow enough or start to look like good candidates for open sourcing,
but this should be an easy enough place to put things without too much overhead.

# Frontend Best Practices

At System initiative, we follow the following best practices to write clean, consistent, maintainable code -
 - Each Vue component is written using the Composition API with the `<template>` section at the top, then the `<script>` sections, and finally (if necessary), the `<style>` section.
 - Whenever possible, all CSS styles should be written as Tailwind classes within the `<template>` section, not in the `<style>` or `<script>` sections.
	 - Legacy Less CSS code in the codebase should be converted into Tailwind styles whenever possible.
	 - Avoid creating computed properties in the `<script>` section to hold classes unless the same classes will be used across many locations in the template.
	 - CSS styles which are impossible to implement using Tailwind can be done in `style` tags, though if you plan to reuse a style you should create a Tailwind-like class for it (see for example, the two `bg-caution-lines` classes).
 - Avoid combining a `v-for` and a `ref` on the same element in a `<template>` whenever possible - usually such a scenario calls for a subcomponent to be made.
 - Avoid using numeric Tailwind classes (`p-2 w-9 text-[14px]`) in favor of semantic classes (`p-xs w-lg text-sm`) whenever possible. Semantic Tailwind classes help ensure consistency across the interface.
 - When adjusting spacing between elements, avoid single direction padding/margin classes (`pl-sm mb-lg`) when possible. Usually there is a way to group such adjustments onto one element or do the adjustments another way (such as adding a `gap` to a flex element).
 - In order to make managing large amounts of Tailwind classes within the template easier, use `clsx()` which is a utility function used to construct className strings conditionally. You can learn more about it here - https://github.com/lukeed/clsx
 - The vue library contains a few utils files you should know about!
	 - `theme_tools.ts` contains multiple helper functions to manage dark/light mode - most importantly, `themeClasses()` is a helpful function that allows for easy composition of the two distinct styles for light and dark mode. It is always preferable to use `themeClasses()` instead of the Tailwind light/dark mode system.
	 - `color_utils.ts`contains helper functions which allow for easier and faster applications of semantic color classes. Make sure your use case matches the standard cases laid out by the TONES object!
- Only use the approved abstract colors - `shade`, `neutral`, `action`, `success`, `warning`, and `destructive`. Do not use named Tailwind colors like `red` or `gray`.
- You can reference all of the current icons, svgs, semantic sizes, and colors which are in the front end on the `Debug Design Reference Page` which can be accessed through the `Dev Dashboard`.

## Code Examples

### Instead of this -
```
<template>
  <div class="mycoolclass">very cool</div>
</template>

<script>
.mycoolclass {
  width: 100%;
  cursor: pointer;
}
</script>
```

### Do this!
```
<template>
  <div class="w-full cursor-pointer">very cool</div>
</template>
```

---

### Instead of this -
```
<template>
  <div class="flex flex-col p-2">
    <div class="pb-2">numeric tailwind classes</div>
    <div class="pb-2">with bottom padding on each div</div>
    <div class="pb-2">so much extra code</div>
    <div class="pt-2">and one could be wrong!</div>
  </div>
</template>
```

### Do this!
```
<template>
  <div class="flex flex-col gap-xs p-xs">
    <div>semantic sizing instead - xs</div>
    <div>now these divs do not need classes</div>
    <div>less code</div>
    <div>and less ways to make mistakes!</div>
  </div>
</template>
```

---

### Instead of this -

```
<template>
  <div
    class="flex flex-row items-center rounded-full border text-xs p-xs cursor-pointer"
    :class="
      selected
        ? 'bg-action-500 dark:bg-action-400 border-action-500 dark:border-action-400'
        : 'hover:text-action-400 hover:dark:text-action-300 hover:border-action-400 hover:dark:border-action-300 border-action-700 dark:border-action-200'
    "
  >
    wow that is a lot of classes and it is overwhelming! plus the conditional part is separated awkwardly
  </div>
</template>
```

### Do this!

```
<template>
    <div
      :class="clsx(
      'flex flex-row items-center p-xs',
      'text-xs rounded-full border cursor-pointer',
      selected
        ? themeClasses('bg-action-500 border-action-500', 'bg-action-400 border-action-400'),
        : themeClasses('border-action-700 hover:text-action-400 hover:border-action-400', 'border-action-200 hover:text-action-300 hover:border-action-300')',
      )"
    >
      now it is much easier to see the classes and understand what's going on! and themeClasses() cleans up the light/dark mode style differences
    </div>
</template>
```