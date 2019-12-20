# System Initiative

This is the source for the System Initiative.

## Quick Start

### Linux 

This repository is known to only work on Arch Linux or Ubuntu. If you're 
trying to run it on something else... sorry, it's not supported.

If you don't have a Linux VM handy, you can use the scripts in the ./scripts/ folder to get yourself a docker container. Install docker, then run the script, and you'll have the repo mounted inside a container. Follow the instructions for bootstrapping, and you're gtg.

### Bootstrapping

To get ready to run this repository, you should run:

```
./components/build/bootstrap.sh
```

This will detect either Arch or Ubuntu, and install the pre-requisites
needed to build a component.

Next, you should run:

```
make build
```

This will ensure that all the pre-requisites for each component are 
installed, and compile each component. If this is successful, 
congratulations, you're all done. 

## Layout

The `components` directory has all the components. 
