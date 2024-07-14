# Function types

Since everything in System Initiative is a result of Typescript function
execution, let’s learn more about what types of Functions exist and how they are
used.

## Action Functions

Action Functions must be defined whenever your Asset is a mirror of a real-world
resource that needs to be created and destroyed. There are three types of Action
Functions: Create, Delete, and Refresh – which subsequently allow you to define
how resources are created, deleted, and refreshed in SI. (Refresh here means ‘go
get me the latest information about my resource from wherever it lives’)

## Qualification Functions

This should sound familiar, but qualification functions allow you to test and
validate aspects of the entire Asset’s configuration. You saw multiple
qualification failures in Tutorial 1 (Deploy a containerized web application),
which were resolved when your Assets were configured properly!

## Code Generation Functions

Code Gen Functions are used whenever you need to generate code to be used
elsewhere in System Initiative. For example, this is how we generated the JSON
required to pass to the AWS CLI to get the ImageId for an AMI in Tutorial 2
(Customizing System Initiative).

## Attribute Functions

Every value on the property tree is the output of a function called an Attribute
Function. By default, each property will have a built-in Attribute Function
configured which simply returns what has been configured in the expected type
(si:setString, si:setObject, etc.). You can configure custom Attribute Functions
to inject logic or transform the shape of your configuration data to meet your
needs. For example, we use an Attribute Function attached to an AWS Ingress
Asset to transform the exposed ports on a Docker Image’s configuration into the
‘IpPermissions’ structure required by an AWS Ingress Rule.
