// import { Node} from "../Node";

// server:
//   name: si-graphql
//   instanceType: size,
//   - securityGroups: group.name
//   ami: ami,
//   keyName: ssh_key.id,
//   userData: userData,
//   tags:
//     use: "si-graphql"

abstract class ServerProperties {
  abstract name: string;
  abstract type: string;
  abstract image: string;
  abstract tags: string;
}

class AwsServerProperties {
  name: string = "blah";
  instanceType: string = "blah";
  securityGroups: string[] = ["blah"];
  ami: string = "blah";
  keyName: string = "blah";
  userData: string = "blah";
  tags: string = "blah";
}

// class Server extends Node {
class Server {
  properties: string = "blah";
  inputs: string[] = ["blah"];
  output: string = "blah";

  id: string = "blah";
  parents: string[] = ["blah"];
  childs: string[] = ["blah"];

  constructor() {}
}
