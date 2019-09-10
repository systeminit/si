import { OperatingSystem } from "@/index";

// The rules:
//
// * A find function must receive enough constraints to satisfy the action that is called in the end
// * A create function operates like a find function, but returns only the entity requested
// * When a component has similar/identical input requirements, then it is portbale. When it is not, we need to prompt you to fill in the needed requirements.

//OperatingSystem({ platform: "Ubuntu", platformVersion: "18.04"})
//.create({
//  name: "mybox", 
//  server: Server({ cpuCores: 4, cpu: Cpu({ architecture: "x86_64" }), },
//});
//

const uuidRegexp = new RegExp(/^[0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12}$/i);

test('Pulumi port', async () => {
  const siGraphql = Service({ 
    name: "si/si-graphql",
    ports: [
      { protocol: "tcp", port: 4000 },
    ],
    stopHook: `hab svc stop si/si-graphql`,
    startHook: `hab svc start si/si-graphql`,
    installHook: `
groupadd hab
useradd -g hab hab
curl https://raw.githubusercontent.com/habitat-sh/habitat/master/components/hab/install.sh | bash
hab license accept
hab pkg install core/hab-sup
cat <<-EOH > /etc/systemd/system/habitat.service
[Unit]
Description=The Chef Habitat Supervisor

[Service]
ExecStart=/bin/hab sup run
Environment=HAB_AUTH_TOKEN=F

[Install]
WantedBy=default.target
EOH
systemctl enable habitat
systemctl start habitat
echo "Waiting to start the service while the sup spins up"
sleep 30
hab svc load si/si-graphql --channel unstable --strategy at-once
`
});
  const couchbase = Service({ 
    name: "couchbase",
    dependsOn: [
      OperatingSystem.find({platform: "Ubuntu", architecture: "x86_64", platformVersion: "18.04"}).dependsOn([
        Package.find({name: "perl"}), 
        Package.find({name: "curl"}), 
        Package.find({name: "shasum"}),
      ]),
      Server.find({memoryGIB: [ ">", "4" ], cpuCores: [ ">", "4" ]}),
    ],
    installHook: `
curl -o couchbase.shasum https://packages.couchbase.com/releases/6.5.0-beta/couchbase-server-enterprise_6.5.0-beta-ubuntu18.04_amd64.deb.sha256
perl -pi -e 's/(.+)/\\1 couchbase.deb/g' couchbase.shasum
curl -o couchbase.deb https://packages.couchbase.com/releases/6.5.0-beta/couchbase-server-enterprise_6.5.0-beta-ubuntu18.04_amd64.deb
sha256sum -c couchbase.shasum
dpkg -i ./couchbase.deb
echo 'Sleeping for couchbase start'
sleep 30
/opt/couchbase/bin/couchbase-cli cluster-init -c 127.0.0.1 --cluster-username si --cluster-password bugbear --services data,index,query,fts,analytics --cluster-ramsize 2048 --cluster-index-ramsize 1024 --cluster-eventing-ramsize 1024 --cluster-fts-ramsize 1024 --cluster-analytics-ramsize 1024 --index-storage-setting default

/opt/couchbase/bin/couchbase-cli bucket-create --cluster 127.0.0.1 --username si --password bugbear --bucket si --bucket-type couchbase --bucket-ramsize 2048 

/opt/couchbase/bin/cbq -engine http://localhost:8091 -u si -p bugbear --script "CREATE PRIMARY INDEX ON \`si\`"
`
});

  const cb = Service.create({ name: "couchbase" }).dependsOn(Server.find({memoryGIB:[ ">", "8" ]}).deploy();
  Service.create({ name: "si-graphql", runningOn: cb.runningOn }).deploy();

  Application.create({ 
    name: "reconcile",
    dependsOn: [
      Service.find({ name: "recapp", type: "windowsService", 

                   buildHook: `...`,
                   installHook: `...`,
                   ports: [ { protocol: "tcp", port: 8080 } ],
      }).dependsOn(OperatingSystem.find({ platform: "Windows" })),
    ]
  });

  Application.find({ name: "reconcile" }).overlay(Server.find({datacenter: DataCenter.find({ name: "detroit-massive"})));


  Server.find({ manufacturer: "HPE", model: "DL380", dependsOn: [DataCenter.find({ name: "detroit-massive" })]}
             ).dependsOn(Application.find({ name: 'VmWare' })).deploy();


  Server.findOrCreate({
    memoryGIB: [ ">", "4" ],
    operatingSystem: OperatingSystem.find({
      platform: "Ubuntu",
      platformVersion: "18.04", 
      services: [siGraphql, couchbase, Service.find({name: "sshd"}), Service.find({name: "ntpd"})]}),
  },
  ).create();

  LoadBalancer.findOrCreate({
    name: "AWS Application Load Balancer",
    targetGroup: siGraphql.instances(),
  }).create();
});

  

test('OperatingSystem Lookup', async () => {
  Server({operatingSystem: OperatingSystem({ platform: "ubuntu", platformVersion: "18.04" })).create();
     Laptop({operatingSystem: OperatingSystem({ platform: "ubuntu" })}).create();

  OperatingSystem.findOrCreate({ 
    platform: "Ubuntu",
    platformVersion: "18.04",
    runningOn: Server({name: "AWS t2.large"}),
  }).update();
 
  OperatingSystem.find({ 
    platform: "Ubuntu",
    platformVersion: "18.04",
    runningOn: Server({name: "AWS t2.large"}),
  }).update();

  OperatingSystem.create({ 
    platform: "Ubuntu",
    platformVersion: "18.04",
    runningOn: Server({name: "AWS t2.large"}),
  }).update({convergent: true});

  Service(
    {
      name: "si-graphql", 
      runningOn: OperatingSystem(
        {
          platform: "Ubuntu", 
          platformVersion: "18.04", 
          //    runningOn: Server({name: "AWS t2.large"}),
        }
      ),
    }).deploy();

    const runningOn = [ Server({name: "AWS t2.large"}), FunctionPlatform({name: "AWS Lambda"}), ];
    for (let r of runningOn) {

  Service(
    {
      name: "si-graphql",
      runningOn: r,
    }).deploy()
  }
  

  os.Service({ name: "ntp", type: "system"}).enable().start();

  const r = await OperatingSystem({ platform: "Ubuntu", platformVersion: "18.04" });
  expect(r.id).toMatch(uuidRegexp);
});
