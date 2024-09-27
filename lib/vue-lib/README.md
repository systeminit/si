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