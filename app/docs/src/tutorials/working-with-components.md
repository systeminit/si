# Working with Components

This tutorial will teach you how to work more deeply with [components](/reference/vocabulary/) in System Initiative.

To follow along, you should:

1. Finish the [Getting Started tutorial](./getting-started), to have basic knowledge of System Initiative.

2. You should have your System Initiative workspace open in another window.

## Create a change set

![Create a change set](./working-with-components/create-a-change-set.png)

Click the create change set button.

Name your new change set `Exploring Properties`, and click the `Create change set` button.

## Add a Docker Image component and set its properties

![Add a Docker Image component and set its properties](./working-with-components/add-a-docker-image-component-and-set-its-properties.png)

Click on `Docker Image` from the `Docker` category of the asset pallete, and drop it inside your workspace.

Name your Docker Image component `nginx`.

## Observe the Docker Images Qualification

![Observe the Docker Images Qualification](./working-with-components/observe-the-docker-images-qualification.png)

Notice the red hexagon in the lower right of your `nginx` Docker Image component. This is the [qualification](/reference/vocabulary#qualification) icon - it is warning you that your Docker Image component is misconfigured.

:::tip
You can also see a components qualification status in the Diagram Outline, the Component panel, and the Qualifications sub-panel.
:::

## Investigate the Docker Images Failing Qualification

![Investigate the Docker Image](./working-with-components/investigate-the-docker-images-failing-qualification.png)

Click the red hexagon on you `nginx` Docker Image component. The properties panel will change to the `Qualifications` sub-panel.

You will see that your Docker Image is failing the `Docker Image Exists` qualification, with the error message 'no image available'.

## Fixing the failing Qualification

![Fixing the Failing Qualification](./working-with-components/fixing-the-failing-qualification.png)

Select the `Attributes` sub-panel.

Set the `image` property to `nginx`.

The qualification icon will then turn green, confirming that a Docker Image named `nginx` exists in the Docker Hub.

## Add a Butane component and set its properties

![Add a Butane component](./working-with-components/add-a-butane-component-and-set-its-properties.png)

[Butane](https://coreos.github.io/butane/) is a configuration file format used by the [Fedora CoreOS](https://fedoraproject.org/coreos/) operating system.

Click on `Butane` from the `CoreOS` category of the asset pallete, and drop it inside your workspace.

Name your Butane component `Web Server Config`.

## Observe the systemd/units property of the Butane component

![Observe the systemd units](./working-with-components/observe-the-systemd-units-property-of-the-butane-component.png)

The `systemd/units` property of the `Web Server Config` takes an array (as indicated by the `[ ]` symbol in front of it). It is currently empty, and configured to be automatically set via a socket.

## Manually setting the systemd/units property

![Manually setting the systemd units property](./working-with-components/manually-setting-the-systemd-units-property.png)

To manually set a property that would otherwise be configured by a socket, click the `set` dropdown, and select `manually`.

Click the `Add array item` button to add an entry titled `unit[0]` to the `systemd/units` array.

Set the `name` property of `unit[0]` to `manual-unit`.

:::tip
Notice that the `Web Server Config` is now failing its qualification, as this is not a valid name for a Systemd unit file!
:::

## Deleting the system/units/unit[0] entry

![Deleting the System/units/unit0 entry](./working-with-components/deleting-the-system-units-unit-0-entry.png)

To delete the `unit[0]` entry, click the trash can icon in the header.

## Connect the Docker Image component to the Butane component

![Connect the Docker Image to Butane](./working-with-components/connect-the-docker-image-component-to-the-butane-component.png)

Switch the systemd/units property to be set `via socket`.

Connect the `Container Image` output socket of your `nginx` Docker Image component to the `Container Image` input socket of your `Web Server Config` Butane component.

## Observe the new system/units/unit[0] entry

![Observe the new system/units entry](./working-with-components/observe-the-new-system-units-unit-0-entry.png)

The new `unit[0]` entry is now set via a function, as indicated by the `f(x)` icon.

The `name`, `contents`, and `enabled` properties have their values inferred by the configuration of your `nginx` docker image.

## Viewing large properties

![Viewing large properties](./working-with-components/viewing-large-properties.png)

Some properties, such as `contents`, are too long to display in the panel. You can hover over the field and then click the icon to have them pop-out to a modal for easy viewing.

## Viewing generated code

![Viewing generated code](./working-with-components/viewing-generated-code.png)

Click the `Code` sub-panel to see the JSON code as it would be processed by Butane.

## Apply the Change Set

![Apply the Change Set](./working-with-components/apply-the-change-set.png)

Press the Escape key, or click on the background of the canvas, to ensure the workspace itself is selected.

Click the `Apply Change Set` button.

Click the `Apply Changes` button in the modal to accept.

## Create another Change Set

![Create a new Change Set](./working-with-components/create-another-new-change-set.png)

Click the `Create change set` button.

Name your new change set `Exploring Part 2`, and click the `Create change set` button.

## Add an exposed port to your Docker Image

![Add an exposed port to your Docker Image](./working-with-components/add-an-exposed-port-to-your-docker-image.png)

Click the `Add array item` button for the `ExposedPorts` property of your `nginx` Docker Image component.

Set the `[0]` value to `80/tcp`.

## Check the Diff for your Docker Image and Butane components

![Check the Diff for your Docker Image and Butane Components](./working-with-components/check-the-diff-for-your-docker-image-and-butane-components.png)

Click the `Diff` sub-panel for your `nginx` Docker Image component. You'll see the currently set properties for the component, and a visual diff of the changes made in this change set (compared to the values on HEAD).

Click your `Web Server Config` Butane component, and you'll see that the entire systemd unit files contents have been updated.

## View the Debugging information

![View the Debugging information](./working-with-components/view-the-debugging-information.png)

Click the `Debug` sub-panel for your `nginx` Docker Image component. Here you will find detailed debugging information about the selected components attributes, input sockets, and output sockets. This information is often useful when customizing or debugging System Initiative.

## Clean Up

![Clean up](./working-with-components/clean-up.png)

Clean up your workspace by highlighting the `nginx` Docker image and the `Web Server Config` Butane components. Press the `Delete` key on your keyboard.

Click the `Confirm` button in the dialog to acknowledge you want to delete these two components.

Click the `Apply Change Set` button. Your `nginx` and `Web Server Config` components are now removed from your workspace.

## Congratulations

You've explored the properties panel! You learned how to:

- Investigate failing qualifications
- Change if properties are set automatically by sockets or manually
- Adding items to arrays and maps
- Delete items from arrays and maps
- View large properties in a modal
- See generated code for a component
- View the diff between a component on a change set and head.
- View detailed debugging information about your components
