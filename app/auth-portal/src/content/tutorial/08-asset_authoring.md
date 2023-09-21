---
title: Asset Authoring
---

## How to create an Asset in System Initiative

Today, we are going to build a System Initiative Asset that allows us to create and manage
[AWS EBS Volumes](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ebs-volumes.html). We are going to create a 100GB EBS Volume in AWS Ohio (us-east-2a)
that has 3000 iops, running on an SSD and is unencrypted.

**Please take care to follow the guide precisely; there's a lot of rough edges. Specifically,
ensure you stick with the function naming scheme and setting the appropriate properties of
attached functions.**

### Anatomy of an Asset

An Asset is a collection of functions that come together to provide the capability to talk
to an infrastructure provider, e.g. an AWS EBS Volume. An Asset is made of the following
types of functions:

* Schema Variant Definition
* Qualification
* Action
* Code Generation
* Attribute
* Validation

The type of functions that an Asset needs will depend on what the Asset does. For example,
if we want our Asset to have a real world representation of our 'digital twin', then we
need `action functions` to be able to _take action_ with the infrastructure provider. Or, you
will need `attribute functions` if you want to manipulate data going
to the Asset or from the Asset.

### Building the Schema Variant Definition of AWS EBS Volume

A Schema Variant Definition, is a function that allows us to create the structure of the
Asset schema. This definition is, usually, made up of:

* Props
* Input Sockets
* Output Sockets

There are other parts to the schema definition, but these are the most common. You can find the
full schema definition in our [source code](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts).
In System Initiative, we try and model out schema similar to how the AWS ecosystem would model
the resource. I usually use the [AWS CLI Documentation](https://docs.aws.amazon.com/cli/latest/reference/#cli-aws)
to understand what the schema needs to include. It's important to note that as we build out Assets, it's easy to
extend the Asset, so you don't need to cover all scenarios in the first creation of the Asset.

Looking at the AWS CLI documentation for [creating an EBS Volume](https://docs.aws.amazon.com/cli/latest/reference/ec2/create-volume.html)
I can see that we will want to have the following properties:

* availability-zone
* region
* iops
* size
* volume-type

#### Authoring in System Initiative

Let's launch System Initiative and go to the `Customize Screen` and create a new changeset to work in.

![customize-screen.png](/tutorial-img/08-asset-authoring/customize-screen.png)

When we click the `New Asset` button, we get a pre-generated code snippet in the function editor. Remove the pre-generated
code and replace it with the following:

_This is the final Asset, we are going to walk through the makeup of it below_

```js
function createAsset() {
    const availabilityZoneProp = new PropBuilder()
        .setKind("string")
        .setName("availabilityZone")
        .setWidget(
            new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build())
        .build();

    const regionProp = new PropBuilder()
        .setKind("string")
        .setName("region")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .setValueFrom(new ValueFromBuilder()
            .setKind("inputSocket")
            .setSocketName("Region")
            .build())
        .build();

    const iopsProp = new PropBuilder()
        .setName("iops")
        .setKind("integer")
        .setWidget(
            new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build())
        .build();

    const sizeProp = new PropBuilder()
        .setKind("integer")
        .setName("size")
        .setWidget(
            new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build())
        .build();

    const volumeTypeProp = new PropBuilder()
        .setKind("string")
        .setName("volumeType")
        .setDefaultValue("gp2")
        .setWidget(
            new PropWidgetDefinitionBuilder()
                .setKind("comboBox")
                .addOption("Magnetic (standard)", "standard")
                .addOption("Provisioned IOPS SSD (io1)", "io1")
                .addOption("Provisioned IOPS SSD (io2)", "io2")
                .addOption("General Purpose SSD (gp2)", "gp2")
                .addOption("General Purpose SSD (gp3)", "gp3")
                .addOption("Cold HDD (sc1)", "sc1")
                .addOption("Throughput Optimized HDD (st1)", "st1")
                .build())
        .build();

    const regionSocket = new SocketDefinitionBuilder()
        .setArity("one")
        .setName("Region")
        .build();

    const volumeIdSocket = new SocketDefinitionBuilder()
        .setArity("one")
        .setName("Volume ID")
        .build();

    return new AssetBuilder()
        .addProp(availabilityZoneProp)
        .addProp(regionProp)
        .addProp(iopsProp)
        .addProp(sizeProp)
        .addProp(volumeTypeProp)
        .addInputSocket(regionSocket)
        .addOutputSocket(volumeIdSocket)
        .build()
}
```

From this code, we can see that a prop uses a `PropBuilder` class and needs a `kind` and a
`name`. It also needs a `widget`. A widget is a way for us to be able to interact with the property via the System
Initiative modelling screen.

You can notice that within the widgets we have combo boxes to help our colleagues.
We have also added a default value for `volumeType`. A volume type is a requirement when choosing
an EBS Volume. AWS defaults to `gp2` in the UI so we can follow suit here in our schema using `setDefaultValue` in the prop.

A prop can get its value from another `prop` or `inputSocket`. We can use the `setValueFrom` class as part of a prop to be
able to do this. We can see an example of this in the `regionProp` in our schema definition.

```js
const regionProp = new PropBuilder()
    .setKind("string")
    .setName("region")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .setValueFrom(new ValueFromBuilder()
        .setKind("inputSocket")
        .setSocketName("Region")
        .build())
    .build();
```

The last parts of the schema definition are the input and output sockets. In System Initiative, we
can model our infrastructure using edges to other assets, including

* a connection between an output socket and an input socket
* a connection between a configuration frame (where all the assets inside the frame are configured by it) 
and an input socket. 

You can see in the schema definition we have added a region input socket, so that it can be configured by a Region frame.

When creating a socket, we need to give that socket an `arity`. The arity is the number of
connections it can make. An EBS Volume can only be in a single region so the region input socket
has an arity of `one`. For my use case, I am going to attach the Volume to a single instance at a
time, so I have made the output socket of volume ID have an arity of `one`.

In System Initiative, we have a loose type system in connecting the Assets. We *currently* use the
name of the sockets to do that, so `Region` as an input socket name matches the `Region` output socket
name of the region Asset.

Now that we have built out the structure of the code, let's update the metadata of the Asset and create it:

![create-asset.png](/tutorial-img/08-asset-authoring/create-asset.png)

When we whack the `Create Asset` button we will get a popup to let us know that we have a new Asset

![create-success.png](/tutorial-img/08-asset-authoring/create-success.png)

We have just built our schema definition. We can now go and attach the behaviour to our Asset to make
it do what we need it to.

### Authoring a Code Generation Function

In order to create an EBS volume, we are going to model the data that we want to send to AWS.
As we will be re-using this data structure, we are going to create a codegen func. Select the Asset we just created
and use the `Attach Function` button and select `New function`. The UI gives us a pop-up and allows us
to select the kind of function.

I've chosen `Code Generation` and set the name to be `awsEc2EbsVolumeJSON` - the name of this function is important as
it will be used as part of the qualification and action functions.

![create-codegen.png](/tutorial-img/08-asset-authoring/create-codegen.png)

When we click the button, we get a pre-generated code snippet in the function editor. Remove the pre-generated
code and replace it with the following:

```js
async function generateCode(input: Input): Promise<Output> {
  const object = {
    AvailabilityZone: input.domain?.availabilityZone,
    Iops: input.domain?.iops,
    VolumeType: input.domain?.volumeType,
    Size: input.domain?.size,
    Encrypted: false, // We are defaulted to non-encryped right now
  };

  return {
    format: "json",
    code: JSON.stringify(object, null, "\t"),
  };
}
```

This code constructs an object and then creates a JSON representation of that object that we can use to pass to the AWS CLI.
Use the `Execute` button to be able to save and update the function attachment.

![codegen-func-attachment.png](/tutorial-img/08-asset-authoring/codegen-func-attachment.png)

*~> Notice, a display name is necessary for the function, so I have made the display name the same as the function name*

We are now able to use this code generation function to be able to qualify and create the resource.

### Authoring a Qualification Function

A qualification function is used to understand if the resource we are attempting to create is going to work as 
expected - like a real time test to give us fast feedback on whether the system we propose is configured correctly.
We can do this using something like a `dry-run` functionality of a CLI. Select the Asset we just created
and use the `Attach Function` button and select `New function`.

I will choose `Qualification` and set the name to `qualifyAwsEbsVolume` and then create it. When we click the button,
we get a pre-generated code snippet in the function editor. Remove the pre-generated
code and replace it with the following:

```js
async function qualification(input: Input): Promise<Output> {
  // This is the name of the Code Generation function we created above!  
  const code = input.code?.["awsEc2EbsVolumeJSON"]?.code;
  if (!code) {
    return {
      result: "failure",
      message: "component doesn't have JSON representation",
    };
  }

  if (!input.domain?.region) {
    return {
      result: "failure",
      message: "component doesn't have a region set",
    };
  }
  
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-volume",
    "--region",
    input.domain.region,
    "--dry-run",
    "--cli-input-json",
    code,
  ]);

  const success = child.stderr.includes("An error occurred (DryRunOperation)");

  return {
    result: success ? "success" : "failure",
    message: success ? "component qualified" : child.stderr,
  };
}
```

This function ensures that we have a JSON representation of our component attributes that we can pass to AWS. It also
ensures that we have a region and then, using the AWS CLI `dry-run` operation, we can ensure that the data we would be
passing to the AWS CLI will do as we expect - in this case, that it could create the EBS volume.

We can use the codegen function that we wrote earlier, and we can shell out to the AWS CLI to be able to qualify that the
component is correct before we try and create it. We can execute that function to update the function mapping and then
the qualification is ready for us to use.

### Authoring Action Functions

Action functions are a way of being able to attach behavior to our components. The types of actions we can do are:

* Create
* Delete
* Refresh

Let's start with authoring create, delete and refresh functions. It's important to set the `entrypoint` of the function attributes
to match the name of the action. When we create the function, we get a pre-generated code snippet in the function editor. Remove the 
pre-generated code and replace it with the following for each of the functions:

![create-action.png](/tutorial-img/08-asset-authoring/create-action.png)

```js
async function create(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }
  
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-volume",
    "--region",
    component.properties.domain?.region,
    "--cli-input-json",
    component.properties.code["awsEc2EbsVolumeJSON"]?.code,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to create EBS Volume, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  return { payload: JSON.parse(child.stdout), status: "ok" };
}
```

This code checks if there is an existing resource in the real world and if it does then we won't try and re-create it. It
then uses the AWS CLI to create the EBS Volume and the data that it passes to the AWS CLI is the result of our code generation
function that we created above.

![delete-func.png](/tutorial-img/08-asset-authoring/delete-func.png)

```js
async function deleteResource(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "delete-volume",
    "--region",
    component.properties.domain.region,
    "--volume-id",
    resource.VolumeId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
        status: "error",
        payload: resource,
        message: `Unable to delete E, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
      };
  }

  return { payload: null, status: "ok" };
}
```

This code will take the Volume ID and the region and call the AWS CLI to delete the volume.

![refresh-func.png](/tutorial-img/08-asset-authoring/refresh-func.png)

```js
async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-volumes",
    "--volume-ids",
    resource.VolumeId,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.exitCode !== 0) {
    console.log(`Volume ID: ${resource.VolumeId}`);
    console.error(child.stderr);
    return {
      payload: resource,
      status: "error",
      message: `AWS CLI 2 "aws ec2 describe-volumes" returned non zero exit code (${child.exitCode})`,
    };
  }

  return { payload: JSON.parse(child.stdout).Volumes[0], status: "ok" };
}
```

This code checks if there is an existing resource in the real world, if there isn't then there's no resource to refresh. It
then uses the AWS CLI to refresh the EBS Volume data from AWS. This refresh function ensures that we are always aware of the
state of the real world resource.

### Let's test it

So we have authored and attached all the functions required to manage our infrastructure. Now we can test it. Let's open the
System Initiative modelling UI and drag an EBS Volume onto the canvas:

![unqualified.png](/tutorial-img/08-asset-authoring/unqualified.png)

We will see that it is initially `unqualified` (if you click on the hex with the red X, you will see why it's so). Let's
update the attributes of the Asset:

![component-attributes.png](/tutorial-img/08-asset-authoring/component-attributes.png)

```text
availabilityZone: us-east-2a
region: us-east-2
size: 100
iops: 3000
volumeType: gp3
```

The function will execute, and we can see that the hex goes to a green check mark. We can use the `Apply Changes` button
to deploy this EBS Volume, and we should see a successful deployment.

![successful-deployment.png](/tutorial-img/08-asset-authoring/successful-deployment.png)

The solid green hex means a successful deployment! Now we can contribute the asset so that others can use it.

### Contribution time

If we go back to the `Customize` screen, the Assets tab should be selected and we should see a `Contribute` button. That
can be used to package up the Asset and share it with System Initiative who will test it and make it part of the available
Asset collection for the community. Let's go ahead and do that:

![contribute-button.png](/tutorial-img/08-asset-authoring/contribute-button.png)

If we are happy, we can whack the button:

![contribution-sent.png](/tutorial-img/08-asset-authoring/contribution-sent.png)

We love our community and are always happy to accept contributions for our Assets!! We are excited to see what Assets that 
you get to make and even more excited to test all the Assets that you want to share with the community.