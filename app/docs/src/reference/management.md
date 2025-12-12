# Management Functions

Management functions are a kind of [function](./function.md) that enables
creation, configuration, and management of other [components](./components.md)
and [subscriptions](./components.md#subscriptions). They provide a programmatic
way to manipulate the graph itself, allowing you to build templates, configure
existing components, import existing infrastructure, manage infrastructure
across multiple environments, or implement custom management workflows.

## How Management Functions Work

Unlike other function types that operate on a single component, management
functions can:

- Create new components
- Update properties of existing components
- Create or remove connections between components
- Modify component positions and hierarchy
- Enqueue or remove actions for components
- Configure multiple components simultaneously

Management functions execute within a change set and return a set of operations
that System Initiative applies to the model.

## When Management Functions Run

Unlike actions or attribute functions, management functions don't execute
automatically - you control when they run.

Management functions are manually triggered when you explicitly run a management
function from the UI, API, or CLI. You might want to run these functions after
configuration changes to the management component (depending on the function's
design), or as a part of automated workflows you define.

## Management Function Arguments

Management functions receive an `Input` argument which is an object that
contains:

- `currentView`: The view in which the function executes (defaults to "DEFAULT")
- `thisComponent`: The component running the management function
- `components`: All connected components, keyed by component ID

This gives the function complete context about the management component and all
components it manages.

The entire structure of the input is:

```typescript
type Input = {
  currentView: string;
  thisComponent: {
    properties: {
      si?: {
        name?: string | null;
        protected?: boolean | null;
        type?: string | null;
        color?: string | null;
        resourceId?: string | null;
      } | null;
      domain?: {
        // Properties based on the type of the component function is attached to
        // ...
      } | null;
      secrets?: {
        credential?: string | null;
      } | null;
      resource?: {
        status?: "ok" | "warning" | "error" | undefined | null;
        message?: string | null;
        payload?: any | null;
        last_synced?: string | null;
      } | null;
      resource_value?: {} | null;
      code?:
        | Record<
          string,
          {
            code?: string | null;
            format?: string | null;
          }
        >
        | null;
      qualification?:
        | Record<
          string,
          {
            result?: string | null;
            message?: string | null;
          }
        >
        | null;
      deleted_at?: string | null;
    };
    geometry: { [key: string]: Geometry };
  };
  components: {
    [key: string]: {
      kind: string;
      properties?: {
        si?: {
          name?: string | null;
          protected?: boolean | null;
          type?: string | null;
          color?: string | null;
          resourceId?: string | null;
        } | null;
        domain?: {
          // Properties based on the type of the component being managed
          // ...
        } | null;
        secrets?: {
          credential?: string | null;
        } | null;
        resource?: {
          status?: "ok" | "warning" | "error" | undefined | null;
          message?: string | null;
          payload?: any | null;
          last_synced?: string | null;
        } | null;
        resource_value?: {} | null;
        code?:
          | Record<
            string,
            {
              code?: string | null;
              format?: string | null;
            }
          >
          | null;
        qualification?:
          | Record<
            string,
            {
              result?: string | null;
              message?: string | null;
            }
          >
          | null;
        deleted_at?: string | null;
      };
      geometry?: { [key: string]: Geometry };
      connect?: {
        from: string;
        to: {
          component: string;
          socket: string;
        };
      }[];
      parent?: string;
    };
  };
};
```

::: warning

The `geometry` properties are deprecated and no longer used.

:::

## Management Function Return Type

Management functions return an object with:

- `status`: Either "ok" or "error"
- `message`: Optional message for the user
- `ops`: Optional operations object containing:
  - `create`: Components to create (keyed by arbitrary ID)
  - `update`: Components to update (keyed by component ID)
  - `actions`: Actions to add or remove (keyed by component ID)

Each operation specifies what should happen to the model when the function
completes.

The entire structure of the return is:

```typescript
type Output = {
  status: "ok" | "error";
  ops?: {
    create?: {
      [key: string]: {
        kind: string;
        properties?: {
          si?: {
            name?: string | null;
            protected?: boolean | null;
            type?: string | null;
            color?: string | null;
            resourceId?: string | null;
          } | null;
          domain?: {
            // Properties based on the type of the component being managed
            // ...
          } | null;
          secrets?: {
            credential?: string | null;
          } | null;
          resource?: {
            status?: "ok" | "warning" | "error" | undefined | null;
            message?: string | null;
            payload?: any | null;
            last_synced?: string | null;
          } | null;
          resource_value?: {} | null;
          code?:
            | Record<
              string,
              {
                code?: string | null;
                format?: string | null;
              }
            >
            | null;
          qualification?:
            | Record<
              string,
              {
                result?: string | null;
                message?: string | null;
              }
            >
            | null;
          deleted_at?: string | null;
        };
        geometry?: Geometry;
        connect?: {
          from: string;
          to: {
            component: string;
            socket: string;
          };
        }[];
        parent?: string;
      };
    };
    update?: {
      [key: string]: {
        properties?: { [key: string]: unknown };
        geometry?: { [key: string]: Geometry };
        connect?: {
          add?: { from: string; to: { component: string; socket: string } }[];
          remove?: {
            from: string;
            to: { component: string; socket: string };
          }[];
        };
        parent?: string;
      };
    };
    actions?: {
      [key: string]: {
        add?: ("create" | "update" | "refresh" | "delete" | string)[];
        remove?: ("create" | "update" | "refresh" | "delete" | string)[];
      };
    };
  };
  message?: string | null;
};
```

::: warning

The `geometry` properties are deprecated and no longer used.

:::

## Common Use Cases

Management functions are commonly used for:

### Importing Existing Resources

Import functions discover resources that already exist in your cloud provider
and create corresponding components in System Initiative with their current
configuration.

For example, an AWS VPC import function would:

1. Query AWS for VPC details using a resource ID
2. Create or update a VPC component with the discovered configuration
3. Switch the component from "create" to "refresh" actions
4. Return the imported configuration

### Creating Templates

Template functions generate complete infrastructure patterns by creating
multiple related components with proper connections and configuration.

For example, a "VPC Template" function might:

1. Create a VPC component
2. Create public and private subnets as children
3. Create route tables and make subscriptions to them
4. Create an internet gateway and make subscriptions to it

### Configuring Multiple Components

Management functions can update many components at once based on policies or
patterns.

For example, a "Tag Manager" function could:

1. Iterate through all connected components
2. Apply a standard set of tags to each
3. Update component properties in bulk
4. Ensure consistent tagging across infrastructure

## Component Creation

When creating components, you specify:

- `kind`: The schema name (like "VPC" or "Route Table")
- `properties`: Initial property values
- `parent`: Optional parent component ID for nested components
- `connect`: Optional connections to create

Each created component is keyed by an arbitrary ID you choose (like "vpc" or
"subnet1") that you can reference in connections or parent relationships.

## Component Updates

When updating components, you specify:

- Component ID as the key
- `properties`: Properties to update (merged with existing)
- `connect`: Connections to add or remove
- `parent`: New parent component ID

## Action Management

Management functions can enqueue or remove actions:

```typescript
actions: {
  self: {
    remove: ["create"],
    add: ["refresh"],
  },
  "component-id": {
    add: ["update"]
  }
}
```

Use "self" to modify actions for the management component itself.

## Management Edges

Components with management functions can have "management edges" to other
component types. These edges define which types of components the function is
allowed to manage, providing type safety and clear boundaries for what each
management function can control.

## See Also

For detailed examples and technical implementation details, see the
[Management Function Examples](/reference/function#management-function-examples)
section in the Functions Reference.
