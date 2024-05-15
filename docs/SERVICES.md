# Services

This document contains the paths and brief definitions of the services that run in the System Initiative software stack.

- **[pinga](../bin/pinga/):** the job queueing and execution service used to execute non-trivial jobs
- **[rebaser](../bin/rebaser/):** where all workspace-level changes are persisted and conflicts are detected based on proposed changes
- **[sdf](../bin/sdf/):** the backend webserver for communicating with `web` that contains the majority of the "business logic"
- **[veritech](../bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments
- **[web](../app/web/):** the primary frontend web application for using the System Initiative software
