---
outline:
  level: [2, 3, 4]
---

# Search Syntax

- Component name: Search for `prod` to find components with prod in their name.
- Schema name: Search for `Instance` to find EC2 instances.
- Combine them: Search for `prod Instance` to find EC2 instances with prod in their name!

When you need more than mere words can convey, you can use more advanced search features like
attribute searches and boolean logic.

## Attribute Search Syntax

To search *inside* components, you can use attribute searches. `InstanceType:`, for
example, will search for instances with that type. Specific syntax for attribute searches:

* **Basic Syntax:** `InstanceType:m8g.medium` will search for m8g.medium instances.
* **Alternatives:** `InstanceType:m8g.medium|m8g.small` will search for m8g.medium or m8g.large instances.
* **Wildcards:** `InstanceType:m8g.*` will search for all m8g instances regardless of size.

  Wildcards can be placed anywhere in the value: `InstanceType:m*.large` will match `m8g.large`, `m7g.large` and even `m7i-flex.large`.

  > Tip: While building your infrastructure, you may want to find things where you did *not* specify an attribute. For example, `!AvailabilityZone:*` will bring back instances where you did not specify an AvailabilityZone, so you can add one!
* **Exact Matches:** `Runtime:"python3.11"` will match only the `python3.11` runtime on a lambda function, but not `python3`.

  You can use quotes (`"`) to pin down your search and match an exact value.  If you don't use quotes, things that *start with* the value you specify are matched.
  
  Quotes will also allow you to use spaces in your search: `Description:"Production Access"`.
* **Attribute Paths:** `LaunchTemplate/Version:1` will match instances with `LaunchTemplate version 1`.
  
  Sometimes an attribute has a generic name, and you need to specify more of its path. `LaunchTemplate/Version:1` is useful because it will *not* bring in every other AWS resource with a random `Version` field set to 1.
* **Schema:** `schema:AWS::EC2::Instance`, or `schema:Instance`, will find all EC2 instances.

All of these features can be mixed and matched: `InstanceType:m8g.*|"mac1.metal"` will find `m8g` instances as well as `mac1.metal` instances.

## Boolean Logic

Sometimes you need more precise logic than just "find things matching A, B and C." For this, we
support full boolean logic, with nesting.

- **Negation:** `!InstanceType:m8g.large` will match all instances that are *not* m8g.large.
- **Alternatives:** `Instance | Image` will match all instances and images.
- **Grouping:** `(prod Instance) | (dev Image)` will match Instances in prod, and images with "dev" in the name.
- "And" (narrowing a search) is done by putting spaces between things. `&` is supported but redundant: `prod Instance` and `prod & Instance` do the same thing.

## Putting It All together

This search will bring back `m8g.medium` instances, and load balancers
with `MaxSize>1`, in prod:

```
prod (schema:Instance InstanceType:m8g.* | schema:LoadBalancer !MaxSize:0|1)
```