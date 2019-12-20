type Component = string;

interface Action {
  name: string,
  inputs?: {
    [key: string]: { 
      componentType: Component,
      multiple: boolean,
      optional: boolean,
    },
  }
}

interface ActionArgs {
  name: string,
  inputs?: {
    [key: string]: Action["inputs"]["componentType"],
  }
}

let serverDelete: Action = {
  name: "delete"
}

let serverCreate: Action = {
  name: "create",
  inputs: {
    "operatingSystem": {
      componentType: "OperatingSystem",
      multiple: false,
      optional: false,
    },
    "sshKey": {
      componentType: "SshKey",
      multiple: false,
      optional: false,
    }
  }
}

let sshKeyCreate: Action = {
  name: "create",
}

// This happens on an agent in the cloud
let s = SshKey.create({name: "killswitch-key"});


// 1. Resolve constraints - "Server" + "constraints" + "inputs" == entity
// 2. Create a new entity of the resolved concrete component
// 3. Call the "create" action on the resolved entity, passing the inputs
//
// 3 can error in a couple ways: First, we can't know in advance what the resolution of our 
// constraint search will be. So it may be that we need more inputs than we provided, and
// hence we wind up returning a "give us more inputs". Otherwise, we can go ahead and do it.
//
// That's not so bad, in particular if you assume this whole thing is an operating system, and
// not something that happens in code all the time this way. Even if it did happen in code at
// runtime, that's not so bad.
//
// Actually, it fails to resolve because we know that all EC2 nodes need SSH keys, and you didn't
// give us any of those. So there is no matching "create" request, much less an entity.
//
// You aren't resolving the enttiy type - you're resolving something that meets your constraints
// and accepts your arguments!!!!
//
// So an EC2 Server Agent would have to expose a "create" action, which accepts an Operating System
// as an input (because it needs a disk image, and we can resolve to that); it would also accept
// a disk image, becuase thats what it needs. That it needs an ssh key is fine. 
//
// Our failure to resolve can send the user searching for a suitable constraint.
Server.create({
  name: "killswitch",
  constraints: {
    "memoryGIB": [ ">", "8" ],
    "cpuCores": 4,
    "cpu": {
      "instructionSet": ["avx-512", "vnni"]
    }
  },
  inputs: {
    operatingSystem: OperatingSystem.find({ platform: "Ubuntu", platformVersion: "18.04" }),
    sshKey: SshKey.create("killswitch-key")
  }
});

// Here, there are no constraints - because this 
Service.create({
  name: "si-graphql",
  constraints: {
    operatingSystem: {
      platform: "Ubuntu",
      platformVersion: [ ">", "18.04" ],
    },
  },
  inputs: {
    source: SourceControl.find({ "integration": Integration.find("github"), repo: "si/si-graphql" }),
  },
  outputs: {
    ports: [
      Port.create({ 
        name: "si-graphql http", 
        inputs: { protocol: "tcp", port: 4000 },
      }),
    ],
  }
});

// Would find the service named si-graphql, and deploy it to the server we made
// earlier. 
//
// This is using a short-hand for the input syntax; if all youa re passing are
// entities that the action uses to make promises about its behavior, you can
// just pass an array.
Service.find({name: "si-graphql"}).deploy([Server.find({name: "killswitch"})]);

// Later, I could pass the Service in to something, or get to the port

Port.find({ name: "si-graphql http" }) // Port{ protocol: "tcp", port: 4000 }

Port.find({ Protocol: "tcp", serviceName: "https" });
Port.find({ protocolName: "http" });

// This would fail unless we found a suitable ServerComponent that accepted a create request 
// with no inputs. 
Server.create({name: "adam"});



