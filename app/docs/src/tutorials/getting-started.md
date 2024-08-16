# Getting Started

<iframe width="560" height="315" src="https://www.youtube.com/embed/7vrIJmP49IE?si=GlRne9jG_l6FXSGX" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

Welcome to System Initiative! This tutorial will teach you how to use System
Initiative to model your infrastructure. We will be deploying a single free-tier
EC2 instance in AWS, and cleaning it up. There should be no cost to you.

To follow along, you'll need three things:

1. [Sign up for System Initiative](https://auth.systeminit.com/signup).

2. An
   [AWS account that allows you to create free-tier resources](https://aws.amazon.com/free/),
   such as EC2 instances.

3. Your System Initiative workspace open in another window.

## Creating a change set

Click the create [change set](../reference/vocabulary#change-set) button.

Name your new change set `Getting started`, and click the `Create change set`
button.

## Add an AWS Credential component

Click `AWS Credential` from the AWS category of the asset pallete, and drop it
on the diagram canvas. This creates a new
[credential](../reference/vocabulary#credential)
[component](../reference/vocabulary#component).

`Resize` the component until it fills most of the visible canvas by dragging the
lower right corner of the frame.

## Name your component

Click on the `AWS Credential` you just added to the canvas. The panel on the
right side of your screen will show the properties of It will have a default
name like 'si-1234'.

Change the name to be `Tutorial`. Pressing enter or clicking outside the textbox
will update the diagram with your new name.

## Add a secret

Click the `Select/Add Secret` button next to the AWS Credential property. Then
click `Add Secret`.

Name your secret `Tutorial Secret`.

Fill in your AWS accounts `Access Key Id` and `Secret Access Key`.
[Refer to the AWS documentation if you do not know what they are](https://aws.amazon.com/blogs/security/how-to-find-update-access-keys-password-mfa-aws-management-console/).

Click `Store Secret` to securely encrypt and save your
[secret](../reference/vocabulary#secret).

:::warning

A side note about [qualifications](/reference/vocabulary#qualification)

If you see a red hexagon in the lower corner of your AWS credential frame at
this point, it means that the credentials are invalid and need to be re-entered.
In some cases, you may need to also add your AWS Session Key.

:::

## Add an AWS Region component and set its properties

Click on the `Region` from the AWS category of the asset pallete, and drop it
inside your `tutorial credential` frame.

Resize the region to fill the space inside the `tutorial credential` frame.

Name your region `North Virginia`.

Set the region property to `us-east-1`.

## Add an AWS EC2 Key Pair and set its properties

Click on the `Key Pair` from the AWS EC2 category of the asset pallete, and drop
it inside your `North Virginia region` frame.

Name your key pair `si-tutorial`.

Set the KeyName property to `si-tutorial`.

## Add an AWS EC2 Instance and set its properties

Click on the `EC2 Instance` from the AWS EC2 category of the asset pallete, and
drop it inside your `North Virginia region` frame.

Name your EC2 Instance `si-tutorial`.

Set the InstanceType property to `t2.micro`.

## Connect the Key Pair to the EC2 Instance

Click the `Key Name` output socket of your `si-tutorial Key Pair` and connect it
to the `Key Name` input socket of your new EC2 Instance component by dragging
the line between them.

## Add an AWS EC2 AMI component and set its properties

Click on the `AMI` from the AWS EC2 category of the asset pallete, and drop it
inside your `North Virginia region` frame.

Name your AMI `Fedora CoreOS`.

Set the ImageId property to `ami-068493ac5013f0936`.

## Connect the AMI to the EC2 Instance

Connect the `Image ID` output socket of your AMI component to the `Image Id`
input socket of your EC2 Instance component.

## Apply the change set

Press the Escape key, or click on the background of the canvas, to ensure the
workspace itself is selected.

You will see two actions enqueued in the right hand panel - one to create the
Key Pair, and the other to create your EC2 Instance.

Press the `Apply Change Set` button.

You'll be prompted with a dialog to confirm you want to take these two actions.
Press the `Apply Changes` button in the dialog to confirm.

## Create the Key Pair and EC2 Instance resources

Applying the change set redirects you to the `HEAD` change set, and enqueues
your actions. The proposed changes panel on the right side of the screen shows
your two pending actions. As the actions are run on AWS, their resulting
[resources](../reference/vocabulary#resource) are added to each model. As this
happens, the actions will disappear from the proposed changes list.

Once both actions have been run, you'll see some confetti, and the changes panel
will be empty.

## Review the si-tutorial EC2 Instances resource data

Select the `si-tutorial EC2 Instance`. Then select the `Resource` sub-panel on
the right side panel. You will see all the information about the EC2 Instance we
created in AWS.

Congratulations! You have created your first resources with System Initiative.

## Clean up

Create a new change set called `Cleanup`.

Select the `Tutorial AWS Credential` component. Press the delete key.

You'll be presented with a dialog confirming you want to delete the components
we created previously. Click Confirm.

Press the escape key or click on the canvas background to select the workspace.

Click the Apply Change Set button to delete your EC2 Instance and Key Pair.
Confirm you want to apply the change set.

After the two delete actions are run, you will have a blank workspace, and no
more resources running in AWS.

## Congratulations

Congratulations - you've created your first resources with System Initiative.
You learned how to:

- Create new change sets
- Add a credentials and secrets
- Add components to the diagram canvas
- Configure components by setting their properties
- Connect components input and output sockets to dynamically configure them
- Execute actions and create resources by applying a change set

## Next Steps

You can:

- Follow the [Create Assets](./creating-new-assets.md) tutorial to learn about
  how to program and extend System Initiative
- Use your knowledge of AWS and System Initiative to learn how to create [AWS
  VPCs]
- Join us on [Discord](https://discord.com/invite/system-init) to ask any
  questions you may have
