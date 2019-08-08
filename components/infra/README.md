# Infrastructure!

Creates all the infrastructure for running the System Initiative. 

Uses Pulumi. 

Right now, it:

* Creates a new container registry
* Creates a tiny EC2 instance
* Install habitat on it
* Installs the latest package from the unstable channel, with an ad-hoc update strategy
* Creates an ALB that uses HTTPS for the API itself
