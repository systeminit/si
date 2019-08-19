
// Assuming you had some kind of catalog of things that already exist
let os = si.operatingSystem.lookup("Ubuntu", { version: "18.04" });
os.update(); // Sorry, nothign to update

let network = si.network.lookup("10.0.0.0/4");
let service = si.service.lookup("si-graphql");
let server = si.compute.lookup(integration: "AWS", instanceType: "x1.large") // << 
  .operatingSystem(os)
  .network(network)
  .service(service);

server.create(); // Implies service, network, and OS behavior
server.start();

// You could also do this
service.start(); // This would imply everything as well - that the server, network, operating system, etc should all exist

//
network.addCompute(integration: "AWS", isntanceType: "x1.large");
network.create();

network.operatingSystems.lookup("Ubuntu").update();

operatingSystems.lookup("Ubuntu").compute(integration: "AWS").update();

service.lookup("si-graphql").map(s => s.upgrade());

new si.Service("si-graphql", service => {
  service.version = "0.0.1",
  service.start = "systemctl start foo";
  service.enable = "systemctl enable foo";
  service.operatingSystem("Ubuntu", { version: "18.04" });
});

// Initial, most verbose way
let os = new si.operatingSystem(o => {
  o.operatingSystemName("Ubuntu");
  o.version("18.04");
  o.integration("AWS").ami("..");
  o.integration("GCP").diskImage(".."); 
});

os.bundle() // <- This would fail, because we have no idea how to do that
os.start() // <- This would fail, because we have not connected it to some compute
os.addCompute(si.compute.lookup(memory: "8gb", cpuCount: "2", vCpu: "2000mips"));
os.start() // <- this would succeed, because we can infer that the compute above should be created!

// CLI
si operatingSystem "Ubuntu" action update
si service "si-graphql" action restart
si service "ntpd" action start 

// Switch to GCP
let service_aws = si.service.lookup("si-graphql", integration: "AWS");
let service_gcp = service_aws.clone().integration("GCP"); // Fails, because we can't map
service_gcp.start();
