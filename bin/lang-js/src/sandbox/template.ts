// deno-lint-ignore-file no-explicit-any

import _ from "npm:lodash";

export default {
  variables,
  updateVarsInViews,
  converge,
  checkUniqueNamePrefix,
  getComponentName,
  sourceOrValue,
};

function sourceOrValue(path: string, thisComponent: any) {
  if (thisComponent.sources[path]) {
    return { "$source": thisComponent.sources[path] };
  } else {
    const lodashPathArray = path.split("/");
    lodashPathArray.shift();
    const value = _.get(thisComponent.properties, lodashPathArray);
    if (value) {
      return value;
    } else {
      throw new Error(`You must provide a value or a subscription for "${path}" in the template components attributes`);
    }
  }
}


interface ComponentPartialName {
  properties: {
    si: {
      name: string,
    }
  }
}

export function checkUniqueNamePrefix(namePrefix: string, components: Record<string, ComponentPartialName>): boolean {
  for (const component of Object.values(components)) {
    const componentName = component.properties.si.name;
    if (componentName && componentName.startsWith(namePrefix)) {
      throw new Error(`Found a component named '${componentName}' that starts with the prefix '${namePrefix}'. Change the 'Name Prefix' so they do not overlap and try again.`);
    }
  }

  return true;
}

type ComponentWithSiNameAttributeValue = string | { '$source': any } | bool | number;
interface ComponentWithSiName {
  attributes: {
    [path: string]: ComponentWithSiNameAttributeValue;
  }
}

export function getComponentName(component: ComponentScaffold): string {
  const siName = component.attributes["/si/name"];
  if (_.isString(siName)) {
    return siName;
  } else if (_.isBoolean(siName)) {
    return `${siName}`;
  } else if (_.isNumber(siName)) {
    return `${siName}`;
  } else {
    throw new Error(`the /si/name of the component is not a string; received value:\n\n${JSON.stringify(component, null, 2)}`);
  }
}

export function updateVarsInViews(
  mgmtComponentKind: string,
  currentView: any,
  thisComponent: any,
  components: any,
): any {
  const templateObjectName = thisComponent.properties.si.name;

  const synced: string[] = [];

  const views: Record<string, any[]> = {
    create: [],
  };
  const update: Record<string, any> = {};
  const create: Record<string, any> = {};

  const templateVarsToSync = _.merge(
    _.get(thisComponent, "properties.domain.Template.Default", {}),
    _.get(thisComponent, "properties.domain.Template.Override", {}),
  );
  console.log("template vars to sync", {
    templateVarsToSync,
  });

  for (const view of _.get(thisComponent, "properties.domain.Views", [])) {
    if (!view.Sync) {
      continue;
    }
    synced.push(view.Name);
    const viewName = `${templateObjectName} ${view.Name}`;
    views.create.push(viewName);
    let exists = false;
    // foo
    for (
      const [componentId, componentObject] of Object.entries(
        components as object,
      )
    ) {
      if (componentObject.properties.si.name.startsWith(viewName)) {
        exists = true;
        update[componentId] = {
          properties: {
            domain: {
              Template: {
                Default: templateVarsToSync,
              },
            },
          },
          geometry: componentObject.geometry,
        };
      }
    }
    if (!exists) {
      const viewName = `${templateObjectName} ${view.Name}`;
      const geometry: Record<string, any> = {};
      geometry[viewName] = thisComponent.geometry[currentView];
      create[viewName] = {
        kind: mgmtComponentKind,
        properties: {
          si: {
            name: viewName,
          },
          domain: {
            Template: {
              Default: templateVarsToSync,
            },
          },
        },
        geometry,
      };
    }
  }
  // something else
  const ops: Record<string, any> = {};
  const result: Record<string, any> = {
    status: "ok",
    message: `Updated Values for Views: ${synced.join(", ")}`,
  };
  if (Object.keys(update).length) {
    ops.update = update;
  }
  if (Object.keys(create).length) {
    ops.create = create;
  }
  if (views.create.length) {
    ops.views = views;
  }
  result.ops = ops;
  console.log("results", {
    result,
  });
  return result;
}

export function converge(
  _currentView: string,
  thisComponent: unknown,
  components: Record<string, unknown>,
  specs: {
    properties: Record<string, any>;
    connect: Record<string, any>;
    geometry: Record<string, any>;
  }[],
  updateFuncName?: string,
): any {
  const templateObjectName = _.get(
    thisComponent,
    ["properties", "si", "name"],
    "unknown",
  );

  const status = "ok";
  const message = "Updated Components";
  const ops: Record<string, unknown> = {};
  const update: Record<string, unknown> = {};
  const create: Record<string, unknown> = {};
  const deletes: Array<unknown> = [];
  const actions: Record<string, { add: string[] }> = {};

  // that gets created, or deleted, by a management function. It needs
  // to be stored as metadata that then gets sent in to the
  // function as an optional argument. That will allow us to let
  // names change without things going very wrong

  // For now, the idempotency key is the name. Bewware!

  // Iterate over all the connected components, and check for any that
  // are no longer in our specification, or that represent environment
  // objects, and schedule them for deletion.
  for (const component of Object.values(components)) {
    const idempotencyKey = _.get(component, "properties.si.name");
    const hasSpec = _.find(specs, {
      properties: {
        si: {
          name: idempotencyKey,
        },
      },
    });
    if (hasSpec) {
      continue;
    } else {
      let isEnvironmentComponent = false;
      for (
        const environment of _.get(
          thisComponent,
          "properties.domain.Views",
          [],
        )
      ) {
        const viewName = `${templateObjectName} ${environment.Name}`;
        if (viewName === idempotencyKey) {
          isEnvironmentComponent = true;
        }
      }
      if (isEnvironmentComponent) {
        continue;
      }
    }
    deletes.push(idempotencyKey);
  }

  // Itereate over all desired specs, and check to see if we need to create them
  // or update them.
  for (const desired of specs) {
    const idempotencyKey = _.get(desired, "properties.si.name");
    let currentId = "";
    let current = {};
    for (const [id, ob] of Object.entries(components)) {
      const match = _.isMatch(ob, {
        properties: {
          si: {
            name: idempotencyKey,
          },
        },
      });
      if (match) {
        currentId = id;
        current = ob as object;
      }
    }
    if (currentId) {
      const updateOp: Record<string, any> = {};
      // Compute the update operation
      const currentProperties = _.get(current, "properties", {});
      if (!_.isMatch(currentProperties, desired.properties)) {
        updateOp.properties = desired.properties;
      }
      // All connections will be overriden
      const currentConnections = _.get(current, "connect", []);
      if (!_.isEqual(currentConnections, desired.connect)) {
        const toRemove = _.difference(currentConnections, desired.connect);
        const toAdd = _.difference(desired.connect, currentConnections);
        if (toAdd.length) _.set(updateOp, ["connect", "add"], toAdd);
        if (toRemove.length) _.set(updateOp, ["connect", "remove"], toRemove);
      }
      // Never update Geometry - right now, we switch from absolute on create
      // to "relative" on update. For now, you have to manually position
      // updates.
      //const currentGeometry = _.get(current, "geometry", {});
      //if (!_.isMatch(currentGeometry, desired.geometry)) {
      //  if (currentView === "DEFAULT") {
      //    _.set(updateOp, ["geometry"], desired.geometry);
      //  } else {
      //    _.set(updateOp, ["geometry", currentView], desired.geometry);
      //  }
      //}
      if (Object.keys(updateOp).length !== 0) {
        update[currentId] = updateOp;
        if (updateFuncName) {
          if (_.get(actions, [idempotencyKey, "add"], []).length === 0) {
            _.set(actions, [idempotencyKey, "add"], [updateFuncName]);
          } else {
            actions[idempotencyKey]["add"].push(updateFuncName);
          }
        }
      }
    } else {
      // Create the component from the desired spec
      const name = _.get(desired, "properties.si.name");
      create[name] = desired;
    }
  }

  if (Object.keys(create).length) {
    ops.create = create;
  }
  if (Object.keys(update).length) {
    ops.update = update;
  }
  if (Object.keys(deletes).length) {
    ops.delete = deletes;
  }
  if (Object.keys(actions).length) {
    ops.actions = actions;
  }

  return {
    status,
    message,
    ops,
  };
}

export function variables(
  thisComponent: Record<string, any>,
): Record<string, any> {
  const something = _.merge(
    {},
    _.get(thisComponent, [
      "properties",
      "domain",
      "Template",
      "Default",
      "Values",
    ], {}),
    _.get(thisComponent, [
      "properties",
      "domain",
      "Template",
      "Override",
      "Values",
    ], {}),
  );
  return something;
}
