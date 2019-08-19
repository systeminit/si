# Purpose

This doc describes the high level ubiquitous language for SI. It's intended to be always evolving.

## Users

Users of the system. 

## Workspaces

Workspaces are where applications get described.

## Integrations

Integrations are where the concrete providers for a given set of components live.

## Components

A component 

### Compute <-- I hate this name

#### Actions

* Start
* Stop
* Restart
* Destroy
* Create
* Console

#### Properties

* networks[]
* storages[] // Man, what an awful plural
* operatingSystems[]
* instanceType
* cpu
* memory
* gpu
* currentState (started/stopped/restarting/destroyed/created..)

#### Implementations

* AWS EC2

### Network

#### Actions

* Destroy
* Create

#### Properties

* computes[]
* loadBalancers[]
* addressRange
* currentState

#### Implementations

* AWS VPC

### Operating System

#### Actions

* Bundle
* Update
* Stop
* Start
* Restart

#### Properties

* computes[]
* cpu
* memory
* gpu
* currentState
* platform
* name
* version

#### Implementations

* AWS EC2
* AWS AMI
* AWS ECS
