# Audit Trail Reference

The Audit Trail feature gives you the ability to know what is going on inside
of your workspace with real-time updates. Through a structured log format and a
sleek, spreadsheet-like interface, you can see mutations to the system and scope
them by entity name, user, change set and more.

## Audit Log Structure

All Audit Logs share the same structure, which includes the entity name, entity
type, kind, metadata and more. However, the metadata field’s structure will
change based on the Audit Log kind. This gives each Audit Log kind the ability
to attach kind-specific information, such as showing the “before value” and
“after value” when updating a property for a component.

Let's look at two examples showcasing the consistent structure with
kind-dependent metadata in action. Here is an example of an Audit Log for a
Component that has been deleted:

```json
{
  // The title that will be displayed.
  "title": "Deleted",
  // The name of the user who caused the event at the time of the event.
  "userName": "nick",
  // The ID of the user who caused the event.
  "userId": "01GW7GQW71JD7B5GV6VBNJBRME",
  // The email of the user who caused the event at the time of the event.
  "userEmail": "nick@systeminit.com",
  // The kind of Audit Log that was created.
  "kind": "DeleteComponent",
  // The type of entity involved in the event.
  "entityType": "Component",
  // The name of the entity involved in the event.
  "entityName": "AWS Credential",
  // The metadata for the event that corresponds to the Audit Log kind.
  "metadata": {
    // An inner field containing the same name as the entity name.
    "name": "AWS Credential",
    // The ID of the Component deleted.
    "componentId": "01JE77FDSNEY9HECH5CRMME2XK",
    // The ID of the SchemaVariant that the Component was based on.
    "schemaVariantId": "01J8FRJB6ESMBMCMM2PBC1SJJY",
    // The name of the aforementioned SchemaVariant.
    "schemaVariantName": "AWS Credential"
  },
  // This is when the event happened.
  "timestamp": "2024-12-03T21:43:04.607739+00:00",
  // The ID of the ChangeSet where the event happened.
  "changeSetId": "01JE77M419EP6P8GVBYKRWWY6S",
  // The name of the aforementioned ChangeSet.
  "changeSetName": "2024-12-03-21:43"
}
```

Now, here is another example for that same Component, but for when one of its
properities was modified:

```json
{
  "title": "Updated Component",
  "userName": "nick",
  "userId": "01GW7GQW71JD7B5GV6VBNJBRME",
  "userEmail": "nick@systeminit.com",
  "kind": "UpdatePropertyEditorValue",
  "entityType": "Property",
  "entityName": "name",
  // All top-level fields have remained the same, except for those inside the
  // metadata field. The inner fields will changed based on the Audit Log Kind.
  "metadata": {
    "propId": "01J8FRJB6ESMBMCMM2PBC1SJK8",
    "propName": "name",
    "afterValue": "AWS Credential",
    "beforeValue": "si-1002",
    "componentId": "01JE77FDSNEY9HECH5CRMME2XK",
    "componentName": "AWS Credential",
    "schemaVariantId": "01J8FRJB6ESMBMCMM2PBC1SJJY",
    "attributeValueId": "01JE77FDSQ30ANJP1021Z0N438",
    "schemaVariantDisplayName": "AWS Credential"
  },
  "timestamp": "2024-12-03T21:40:55.268312+00:00",
  "changeSetId": "01JE77F4E5P1S4228A3P5978NR",
  "changeSetName": "2024-12-03-21:40"
}
```

Let's say that a new field has been added to the metadata for an
`UpdatePropertyEditorValue` event. It is called `containsWhitespace` and
indicates whether or not the `afterValue` contains a whitespace.

While this scenario is a bit contrived, it shows that metadata not only changes
for a given kind, but can also be expanded in the future, so long as the change
is backwards-compatible. Here is an example using the same Audit Logas as above,
but with the new field:

```json
{
  "title": "Updated Component",
  "userName": "nick",
  "userId": "01GW7GQW71JD7B5GV6VBNJBRME",
  "userEmail": "nick@systeminit.com",
  // The kind is the same as the above Audit Log.
  "kind": "UpdatePropertyEditorValue",
  "entityType": "Property",
  "entityName": "name",
  // The metadata is the same as the above Audit Log, except it has a new field.
  "metadata": {
    "propId": "01J8FRJB6ESMBMCMM2PBC1SJK8",
    "propName": "name",
    "afterValue": "AWS Credential",
    "beforeValue": "si-1002",
    "componentId": "01JE77FDSNEY9HECH5CRMME2XK",
    "componentName": "AWS Credential",
    "schemaVariantId": "01J8FRJB6ESMBMCMM2PBC1SJJY",
    "attributeValueId": "01JE77FDSQ30ANJP1021Z0N438",
    "schemaVariantDisplayName": "AWS Credential",
    // This is the new field!
    "containsWhitespace": true
  },
  "timestamp": "2024-12-03T21:40:55.268312+00:00",
  "changeSetId": "01JE77F4E5P1S4228A3P5978NR",
  "changeSetName": "2024-12-03-21:40"
}
```
