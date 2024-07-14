# The doubler

This tutorial brings together in a simplistic way all of the basic building
blocks of an asset into a simple component that takes an input, doubles it, and
makes the result available for other components to consume.

In this tutorial you will learn

- How to make a new component
- How to create an attribute function
- How to bind attributes to sockets

## Steps

1. Create a new change set `Change Set 1`
2. Click on the customize screen
3. Click on the `+` Create new Asset button, enter asset name `Doubler`
4. In the editor, paste the following

```
schema
```

This will create a new component named Doubler which has 2 attributes, 1 input
attribute and and another which contains the computed value. It also has an
output socket which contains the computed value.

5. Change the category to `Doubler`
6. Click `Regenerate Asset`
7. Click the new function button and select attribute function. Set the output
   location to `/root/computed`
8. Add an input parameter `value` of type `string`
9. Click `Edit Bindings` and set the binding to `/root/domain/input`
10. Set the function to the following

```typescript
function main(Input: input) {
  return input.value * 2;
}
```

11. Go back to the modeling screen and drag the new `Doubler` component onto the
    canvas
12. Enter a value in the `input` field to `8`
13. Notice that the `computed` field of the asset is now `16`

## Exercise

Take what you have learned and add a new attribute to the component `tripled`,
and create the associated attribute/binding to triple the asssets input.

_insert screenshot here_
