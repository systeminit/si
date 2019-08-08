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

const size = "t2.small";

const ami = "ami-08fd8ae3806f09a08";

const userData = `#!/bin/bash
apt-get update -y
apt-get upgrade -y

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
Environment=HAB_AUTH_TOKEN=${config.require("habAuthToken")}

[Install]
WantedBy=default.target
EOH
systemctl enable habitat
systemctl start habitat
echo "Waiting to start the service while the sup spins up"
sleep 30
hab svc load si/si-graphql --channel unstable --strategy at-once
`;

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
