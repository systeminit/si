# Free Floating Props and Providers

This document contains an idea generated while working on providers.

## The Question

We can potentially allow internally consuming and internally providing “providers” to exist on free floating props since you are only bound by familial constraints (i.e. parentage relationships on `Props`).
Evaluating within that relationship structure _should be_ possible.
The question is just because we can, does that mean we should?

## Implementation Suggestions

Option A:

- `InternalImplicitProviders` (replace with better name, exist on `Props` or `Schemas` or `SchemaVariants`)
- `InternalExplicitProviders` (externally consuming, exist on `SchemaVariants` only )
- `ExternalProviders` (exist on `SchemaVariants` only)

Option B:

- `InternalProviders` that internally consume (exist on `Props` or `Schemas` or `SchemaVariants`)
- `InternalProviders` that externally consume (exist on `SchemaVariants` only )
- `ExternalProviders` (exist on `SchemaVariants` only)

Option C:

- `Props` can internally provide and internally consume (can happen at any context?)
- `InternalProviders` can only externally consume (exist on `SchemaVariants` only)
- `ExternalProviders` (exist on `SchemaVariants` only)

## Things to Keep in Mind

- `InternalProviders` and `ExternalProviders` _only_ exist in the `SchemaVariant`-specific context, but they can work with data from an arbitrary context
- We do not create providers for things under arrays or maps

## Conclusions

- We likely cannot do this because even though this _could_ work at first, you could eventually be under a map or array
- We may not be able to do this because of cyclic issues
- However, this _might_ be possible someday (let's save this idea for another day)