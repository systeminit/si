# Create or validate the IAM roles are in place

* Make sure you have a role
* That role needs AmazonEKSClusterPolicy and AmazonEKSServicePolicy
* Validate the trust relationship - apparently if you didn't create it?

# Properties
* cluster name
* kubernetes version
* vpc (default!)
  * subnets?
* security groups
  * use the default security group
* server access
  * private
  * public
* logging!
  * api server
  * audit
  * authenticator
  * controller manager
  * scheduler
* tags
  * whatever you want

# Managed Node Group
* name
* iam role with the right shit applied
* subnets
* allow remote access
* ssh key pair
* tags
* k8s labels
* regular or gpu enabled
* instance types
* disk size in GiB (attaches an ebs volume!)
* minimum group size
* maximum size
* desired size

