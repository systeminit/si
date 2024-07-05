# Getting Started

Welcome to System Initiative (SI). It's a powerful new way to manage your infrastructure and applications. This tutorial will get you up to speed on all the basics.

To follow along, you'll need two things:

1. Sign up for System Initiative.

2. An AWS account that allows you to create free-tier resources, such as EC2 instances.

## Workspaces

When you first sign in to SI, you'll be in a new, blank workspace. Workspaces are where everything happens in SI. They are where you model and customize your infrastructure. You can have as many workspaces as you want. You can switch workspaces at any time using the workspace dropdown.

## Change Sets

Everything in System Initiative happens inside a Change Set. If you've used version control systems like Git before, think of a Change Set like a branch. You should currently be on the \`HEAD\` Change Set. HEAD represents the current desired state of your infrastructure. To make any changes, you must be in a new Change Set.

To switch between change sets, use the change set dropdown. To create or abandon them, use the buttons to the right of the drop-down.

Create your first Change Set by clicking the new Change Set button, and name it "Tutorial".

## Components and Resources

In System Initiative, we model infrastructure through components and resources. A component is a theoretical representation of a resource that you want to manage. (Think of the component as a digital twin of the real-world resource it represents.)

## Configuration Diagram

The canvas in the center of your screen is a configuration diagram. You will place the components you need on it, and then connect them together to help configure them. Think of it like building an architecture diagram that also configures its components.

## Creating your first component

We want to create some resources in AWS, so the first component we need is our AWS Credential. In the lower left panel you will see a section titled 'Assets'. Each entry in this panel is a different component you can use. Click in the search box in the assets panel, type 'cred', and you will see AWS Credential. Click that, and then drop it on the diagram canvas to the right.

AWS Credentials appear on the diagram as a frame. Components that are frames configure the components within their boundaries. Frames are resizable. Make your new AWS Credential frame large enough to fill most of the visible diagram.

## Configuring the AWS Credential

Click on the AWS Credential component. You'll see that the right panel changes, displaying information about the component's attributes. Start by giving your component the name 'development'. Giving your components meaningful names makes them easier to find later. It also makes the context contained in the diagram easier to see at a glance.

AWS Credentials are sensitive and secret information. System Initiative encrypts and stores this secret data for you. Click the 'select/add secret' box in the attributes panel, then click 'add secret'.
