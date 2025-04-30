# Newhotness Survival Guide

Hello!
This is [@nickgerace](https://github.com/nickgerace)'s guide to navigating the "newhotness" UI and designing materialized views for it.

## How do I design a good MV for the "Newhotness" UI?

Follow these unordered rules:

1. Obey the merkle tree hash gods
2. Pretend as if the pinia stores do not exist
3. Drive Vue composition through props
4. Limit the number of situations the frontend needs to mash MVs together into a big computed type

### 1) Obey the markle tree hash gods

Lesson learned from [#5914](https://github.com/systeminit/si/pull/5914): do not break out of the `edda` changes iterator and MV kinds loop.
In that PR, we un-did changes that were "too smart" and introduced immediate hops to MV generation.

Here's the problem: if you don't rely purely on the immediate change's `EntityKind` or the `ChangeSetId`, you have no way of handling deleted nodes.
Why?
The `rebaser` sends a request to `edda` with _only_ the changed nodes and the post-rebase snapshot.
The `edda` task only has enough information to use checksums and changes with the current snapshot to generate MVs.
This is a good design in that `edda` isn't smart enough for its own good: it exists in a "post-rebase" world and needs to be as fast as possible at generating MVs.

The takeway: only generate MVs based on the immediate change's `EntityKind` or the `ChangeSetId`.
It is possible that a theorhetical "`edda` 2.0" could handle "hops" to trigger MV generation, but at the time of writing, obey the merkle tree hash gods.

> [!TIP]
> Drive your MV creation and lifecycle by your trigger entity's merkle tree hash calculation.
> Breaking out of the box is a recipe for pain.
> Staying in the box is a path to victory.

### 2) Pretend as if the pinia stores do not exist

This one is pretty self-explanatory.
The new frontend architecture does not use pinia stores.

Will bedrocks stores like the "auth" store stick around for the near future?
Sure, but the "components" store will more than likely not.

We want to limit cross-referencing of data and random data piping into Vue components, which leads us to #3 and #4.

### 3) Drive Vue composition through props

You'll see that even in the root `Workspace.vue`, the `changeSetId` and `workspacePk` props are populated via the URL.
These are passed down to every descendent component.

We can take advantage of the props pattern through queries.
Selected a component on the grid?
Pass it down.
Soon, the days of `thingStore.selectedThingId` will be over.

### 4) Limit the number of situations the frontend needs to mash MVs together into a big computed type

If you follow all aforementioned steps, you should be in a good spot for this one.
However, you may notice that in [#5914](https://github.com/systeminit/si/pull/5914), we have a `computed` usage that partially "links" two MVs.
That decision not only predates formal, weak references, but it was required to "obey the merkle tree hash gods".

Every decision in authoring an MV will be nuanced.
We want to prevent the cross-referential objects store palooza that exists in the current UI.
However, there may be times where, because of the graph structure, you may need more than one MV to solve your problem.
In fact, multiple MVs is totally fine!
It's the cross-referential-big-ass-computed types that can get hairy.
So long as we are cognizant here, we will setup a good foundational for the future.

## I want to change my MV on `main`. What can I do?

Delete the old MV and create a new MV.
MVs and reference kinds are immutable, but they can be renamed, dropped and created, at will.

> [!TIP]
> Keep in mind, this feature is not enabled yet, so if we need to destroy what's in frigg and start over, we will.

## Where are all the crates and libraries that I need to pay attention to?

Let's start with the frontend.

- `app/web/src/newhotness/`: this is the destination for the "newhotness" UI
- `app/web/src/workers/types/dbinterface.ts`: this is where you will store types for your MVs until type generation is added
- `app/web/src/workers/webworker.ts`: this is where you will handle references for your MVs that reference other MVs until type generation is added

Now, onto the backend.

- `lib/si-frontend-types-rs/src/newhotness.rs`: this is where "newhotness" MVs are defined
- `lib/dal-materialized-views`: this is where "newhotness" MV generation logic goes
  - this crate extends the `dal` and is only used by `edda-server` and `dal:test-integration`
  - it is not included in the `sdf` build graph nor does it invalidate the `dal` build cache
  - this crate's placement in build graph means faster builds for everything but `bin/edda` and `lib/dal:test-integration`, which is neat
  - once the "newhotness" term dies, this will be the home for all MV generation logic
- `lib/dal-materialized-views-macros`: this crate contains macro(s) that `edda-server` will use to build MVs
- `lib/dal/tests/integration_test/newhotness`: this is where "newhotness"-specific MVs are tested, which should be coming from `lib/dal-materialized-views`
- `lib/edda-server/src/change_set_processor_task/materialized_view.rs`: where the MVs are generated

## What should I watch out for when developing the "newhotness" UI and MVs?

There are pitfalls to avoid and configurations to set before getting started.

- Ensure the `NEW_HOTNESS` flag is enabled in `app/web`
- Ensure the rebaser has its "generate MVs" feature enabled (or, comment out EVERY location where the "if" statement for the feature exists to ensure it is working)
- Add your MV to the dependency graph at the end of `lib/edda-server/src/change_set_processor_task/materialized_view.rs` because it is NOT a compile time check

## I rebased with latest `main` and have a running stack. What do I do?

Assuming there are no major architectural changes, re-run and rebuild the following, _at minimum_:

- `sdf` for index invalidation, rebuilding and fetching
- `edda` for MV generation
- `web` for new frontend changes

That being said, rebuilding all backend services is recommended as `prod` and `tools-prod` behave in that manner.
If in doubt, tear down the entire stack and start over.
