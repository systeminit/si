//
//  All runs take an 'invocation' key that can be used for idempotency
//
//  Flow:
//
//  - Search for all the baseQueries
//    - create 'baseline' and 'working set'
//  - Run any name transformation regex; updating all the si/names in the working set
//  - Add an si/tags/idempotencyKey that is the unique identifier for the component; must be stable across invocations
//  - Add the si/tags/from-template tag pointing at the name of the template
//  - Add the si/tags/from-template tag pointing at the time the template was run
//  - Run the transformation function, passing all input data, and transforming anything in the working set
//  - Update all inclusive subscriptions in the working set
//  - Create arguments for all non-inclusive subscriptions that are in the working set
//    - any subscription where the propPath is identical, and the target is also identical, gets an identical option
//    - use the input data to resolve if possible
//    - any non-resolved subscriptions default to the component they are currently pointing at
//  - Create a change set from the command line
//  - Search for all components that match the invocation key + template name
//    - For any component that has the same invocation key + template name + idempotencyKey, update any properties that don't match
//    - For any component that does not match, create it
//    - For any component that was in the invocation key + templat name spec but *no longer exists* in the match, delete it.
//  - Summarize the work that was done
//
//  Have a mode where the search results in the baseline set are cached after transformation. This is how you can make the
//  template stable and sharable across workspaces - you export the data to the cache, then run the template with the cache

include * from "si-templates"

changeSet(...)
search([
  "tags: production"
]);

namePattern();

inputs({
  "availibityZone": "us-east-1a"
});

transform((workingSet) => {
  for (x in workingSet) {
    if x.schema == "AWS::Subnet" {
      setAttribute(x, "subnet", "/domain/availabilityZone", inputs["availibityZone"]);
    }
  }
  return workingSet;
});

// si-condiut template run --fromCache data.json --key "production" --input-availaiblity-zone "I like cheese" ./chips.ts
// searching..
// transforming..
// created


