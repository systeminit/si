import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";

// Create a new ECR Repository
const repo = new awsx.ecr.Repository("si");
export const repoUrl = repo.repository.repositoryUrl;

const config = new pulumi.Config("si-graphql-infra");

const eip = new aws.ec2.Eip("si-graphql");

const ssh_key = new aws.ec2.KeyPair("si-key", {
  publicKey: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDZiV25xYzz2cCOMHRxA333VsXXwq8AEZTL0wvrfpvyXtd6r7PeyTD9F+LjjITmKR8kRv6dfRrAIFNl4n/yugFl46Wa9m5aU7R/EkfcEo38j61tC6SywC+JFGuw3XeUxAhVHE6zQYw5esX0AHahOf0IQw8f7YuzDxDocAUiXgRsFDr2aHiMkAISOzWjGimAYnP00rvo//a4u5ogMKZNYZJdjoNudx8qXfT+BR4/UhcsUp2Ky5uaFDGgu6lqnaAzXBold8x8Ag3Fv0OKw8NES+K/tZElzd4k+BR1onEGqksjk7ZPISQuYxhWtVtySfOn3KhaPrM9D2ZSEKhQvqjeuCQyuHytoXyR8xtYNtyFTRlQFVwTvWBELb8kOoWK0NQzYL/pUc71SqANjnSxCSlPeKrbsepR187TVMLawAp92k/ZO/HtUpNemJzn1W+FwNsvYgwaZiTe0GSgrmnNe7AGzAMaBGUrRlhEIRGvQcdXfriu5VA0nWw8RBxrWLSVqf57U5M= admin@systeminit.com"
});

const size = "t2.large";

const ami = "ami-08fd8ae3806f09a08";

const userData = pulumi.all([
  config.requireSecret("siSshKeyServiceConfig"),
  config.requireSecret("siAccountServiceConfig"), 
  config.requireSecret("masterDbPassword"), 
  config.requireSecret("jwtSecret")]
                           ).apply(([
                             siSshKeyServiceConfig,
                             siAccountServiceConfig, 
                             masterDbPassword,
                             jwtSecret]) => 
`#!/bin/bash
apt-get update -y
apt-get upgrade -y

pushd /tmp
curl -o couchbase.shasum https://packages.couchbase.com/releases/6.5.0-beta/couchbase-server-enterprise_6.5.0-beta-ubuntu18.04_amd64.deb.sha256
perl -pi -e 's/(.+)/\\1 couchbase.deb/g' couchbase.shasum
curl -o couchbase.deb https://packages.couchbase.com/releases/6.5.0-beta/couchbase-server-enterprise_6.5.0-beta-ubuntu18.04_amd64.deb
sha256sum -c couchbase.shasum
dpkg -i ./couchbase.deb
echo 'Sleeping for couchbase start'
sleep 30
/opt/couchbase/bin/couchbase-cli cluster-init -c 127.0.0.1 --cluster-username si --cluster-password "${masterDbPassword}" --services data,index,query,fts,analytics --cluster-ramsize 2048 --cluster-index-ramsize 1024 --cluster-eventing-ramsize 1024 --cluster-fts-ramsize 1024 --cluster-analytics-ramsize 1024 --index-storage-setting default

/opt/couchbase/bin/couchbase-cli bucket-create --cluster 127.0.0.1 --username si --password "${masterDbPassword}" --bucket si --bucket-type couchbase --bucket-ramsize 2048 

sleep 20

/opt/couchbase/bin/cbq -engine http://localhost:8091 -u si -p "${masterDbPassword}" --script 'CREATE PRIMARY INDEX ON \`si\`'

apt-get remove -y docker docker-engine docker.io containerd runc
apt-get autoremove -y
apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg-agent \
    software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | apt-key add -
add-apt-repository \
   "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
   $(lsb_release -cs) \
   stable"
apt-get update -y
apt-get install -y docker-ce docker-ce-cli containerd.io
systemctl enable docker
systemctl start docker

echo '${siAccountServiceConfig}' > /etc/si-account-config.toml
echo '${siSshKeyServiceConfig}' > /etc/si-ssh-key-config.toml
echo 'JWT_KEY=${jwtSecret}' > /etc/si-graphql-api-config.env

docker login -u adamhjk -p 0a27ddb56fb70eb2faf5335a43d104ccfc681223 docker.pkg.github.com
docker run -e "DOCKER_VERNEMQ_ACCEPT_EULA=yes" -e "DOCKER_VERNEMQ_ALLOW_ANONYMOUS=on" -p 1883:1883 --restart always --detach --name vernemq erlio/docker-vernemq
docker run --restart always --network=host --detach --name si-graphql-api-service -v /etc/si-graphql-api-config.env:/svc/si-graphql-api/.env docker.pkg.github.com/systeminit/si/si-graphql-api-service:latest
docker run --restart always --network=host --detach --name si-account-service -e NO_SIGNUPS=1 -v /etc/si-account-config.toml:/svc/si-account/config/default.toml docker.pkg.github.com/systeminit/si/si-account-service:latest
docker run --restart always --network=host --detach --name si-ssh-key-service -v /etc/si-ssh-key-config.toml:/svc/si-ssh-key/config/default.toml docker.pkg.github.com/systeminit/si/si-ssh-key-service:latest
docker run --detach --name watchtower -v /root/.docker/config.json:/config.json \
  -e WATCHTOWER_NOTIFICATIONS=slack \
  -e WATCHTOWER_NOTIFICATION_SLACK_HOOK_URL="https://hooks.slack.com/services/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" \
  -e WATCHTOWER_NOTIFICATION_SLACK_IDENTIFIER=watchtower \
  -e WATCHTOWER_NOTIFICATION_SLACK_ICON_EMOJI=:whale: \
  -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --cleanup
`);

const vpc = awsx.ec2.Vpc.getDefault();
const lbsg = new awsx.ec2.SecurityGroup("si-graphql-lb", {
  vpc,
  egress: [
    { protocol: "-1", fromPort:0, toPort: 0, cidrBlocks: [ "0.0.0.0/0" ] },
  ],
});

let group = new aws.ec2.SecurityGroup("si-graphql", {
    ingress: [
        { protocol: "tcp", fromPort: 22, toPort: 22, cidrBlocks: ["0.0.0.0/0"] },
        { protocol: "tcp", fromPort: 4000, toPort: 4000, cidrBlocks: ["0.0.0.0/0"] },
    ],
    egress: [
      { protocol: "-1", fromPort:0, toPort: 0, cidrBlocks: [ "0.0.0.0/0" ] },
    ]
});

const alb = new awsx.lb.ApplicationLoadBalancer("si-graphql-lb", {
  securityGroups: [ lbsg ],
  external: true
});

const tg1 = alb.createTargetGroup("si-graphql-tg", { port: 4000, protocol: "HTTP" });

const httpsListener = tg1.createListener("https-listener", {
  port: 443,
  protocol: "HTTPS",
  // The ARN here must be manually created by the AWS Certificate Manager
  certificateArn: "arn:aws:acm:us-west-1:835304779882:certificate/a3a34e4c-57c0-4e5d-8a9e-b2c5ece487a6",
});


const httpListener = alb.createListener("http-listener", {
    port: 80,
    protocol: "HTTP",
    defaultAction: {
        type: "redirect",
        redirect: {
            protocol: "HTTPS",
            port: "443",
            statusCode: "HTTP_301",
        },
    },
});


let server = new aws.ec2.Instance("si-graphql", {
    instanceType: size,
    securityGroups: [ group.name ], // reference the security group resource above
    ami: ami,
    keyName: ssh_key.id,
    userData: userData,
    tags: {
      use: "si-graphql",
    }
});

tg1.attachTarget("si-graphql-backend", server);

exports.eip = eip;
exports.alb_dns_name = alb.loadBalancer.dnsName;
exports.publicIp = server.publicIp;
exports.publicHostName = server.publicDns;
