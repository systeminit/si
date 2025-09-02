# NEW HOTNESS

<!-- NOTE(nick): added attribution here -->
Author: [@jobelenus](https://github.com/jobelenus)

## PROBLEM STATEMENT

Brittle.

Many of our components are too tightly coupled. We shouldn't have components where when we send a new kind of prop to it, we have to write logic within the component to handle the new prop. We mix layout / visual-only work with business logic. Many of our re-usable components are over-loaded with complexity.

Our use of tailwind is excessive some times, it feels like an incantion that has to have exactly the right repititve elements or it doesn't work. Good examples are, over a dozen utility classes, or when a many succesive divs need flex rules for every single one of them, with slight differences. Or having multiple elements stitch together a border (e.g. zero border bottom on the top element, and zero border bottom on the bottom element, or left/right). I know I have often had to play with colors, and while `themeClasses` is obviously useful, we shouldn't have to pluck a color whenever we implement a new thing.

My impression is that most of the cost of implementing new work on the front end is accidental complexity. Changes have too many indirect side-effects and we scramble to paper them over with more CSS or code &amp; branching logic. Some re-usable components have implicit requirements from their containers (e.g. I've been caught several times where a components uses position absolute or relative which requires a change to the parent, and other changes further up the stack so nothing is broken.)

Despite having many re-usable components, we also have a lot of duplicated logic that we copy and paste around that could be made into useful composables.

Our stores turned into a mess. Our historical choices conflated the real purpose of stores with making HTTP calls. That tight binding was very painful.

## GOALS

Components should do a minimum amount of work. Just enough to get a job done. Use multiple components to split out functionality that can be re-used.

We want to copy & steal things from the original web app we like working with. We want to re-use things that have hard-boundaries.

We should make components that handle *only* layout, and components with business logic should use those layout components. [A good example](https://github.com/systeminit/si/pull/5862/files#diff-2df1de572e9d0bb6f6f2deb8a7d9464c0b1959d430d1e122dff4015eecb6acb0) is splitting `ActionCard` into `ActionCardHistory`, `ActionCard`, and `ActionCardLayout`

We can use some simple global styling that reduces our need for re-declaring classes. We can have color declarations that use `themeClasses` behind the scenes (or something like it) to define the dark and light colors so each engineer doesn't have to do it bespoke each time.

Use tanStack:
- virtualization, for any variable length list that grows over time
- form, for large editing forms (like the component editor), not necessary for one or two inputs, or filters

Stores are *only* to be used when components that are not directly hierarchical *must* rely on the same *shared mutable* state. If the state is not mutable that means it should be a prop, or a provide / inject coming from a shared ancestor. (See: the new `Workspace` has a provide for `workspacePk` and `changeSetId` for any descendant to grab, no prop-drilling needed)

## APPROACH

Discipline. Work in small increments. Get something working. Refactor it into re-usable pieces. Create to give control back to the component, instead of increasing the logic within a component. Example: `VormInput` actually functions really well, and I believe we should re-use it. But imagine how much better it would be if each input type was actually its own component, and `VormInput` had a slot you used it in.

Do not import a component from the original app because changes & fixes in the original app could break us. Copy the component, and trim it down to size for _only_ what we need it to do. (For example: I took the `TextPill` from the app and put it in the `vue-lib` with some enhancements.)

Exceptions apply for some of the basic components (e.g. `NavbarRight`, `NavbarLeft`) and basic stores (`workspaceStore`, `changeSetStore`, `realtimeStore`) that we have to inherit just to get the app boostrapped in the same environment as the original app.

CSS Grid is often times a better alternative than Flexbox, you don't have to has as many containing divs to get things working. Using `grid-template-area` is very helpful for everyone to see the intended grid layout.

I haven't found a reason to use a store yet! And I don't necessarily see one, but maybe, one day. The bifrost architecture and use of tanStack query will be a huge lift &amp; reason we don't need stores.

There are a fair amount of new approaches in here. Look to them for inspiration!

# ACHIEVEMENTS SO FAR
- I believe I fixed the "you could make the app window scroll" and break the layout
- The main body will never grow larger than the space between the navbar and footer
- Re-usable accordion style collapsible (with no extra html, its not even a "component" its just CSS) :chefskiss:
- Scrolling contents (with no extra html)
- Re-usable component for putting instruction text "inside" `VormInput` (stolen and refactored from `FloatingConnectionMenu`)
- Global / re-usable card definition and grid, CSS FTW :metal:
- Placeholder routes for views and secrets
- global keydown watcher and emitter for shortcuts and user actions
- breadcrumbs that give user links back to where theyve been

## THINGS TO AVOID

`TreeFormItem` is an example of a component to avoid. We do not want to port this over as-is. It is not a re-usable component with hard boundaries.
The props and logic are tightly bound and incredibly difficult to work with.

Nesting multiple flexbox items around one another means the answer is a grid.

Deep component trees & long lists of props are a smell that you can leverage slots!

# NEW VERBIAGE

We no longer use the word "edge" in the web. It was overloaded with the graph definition of edge. We shall use the word "connection" instead.

We call the "things in a grid" tiles, not cards. The goal is interaction & exploration, not information density.
