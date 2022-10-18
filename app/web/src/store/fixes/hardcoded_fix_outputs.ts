export default {
  Egress: JSON.stringify(
    JSON.parse(
      '{"Description":"Allows production traffic to the app server","GroupName":"prod-app","IpPermissions":[{"FromPort":443,"IpProtocol":"tcp","IpRanges":[],"Ipv6Ranges":[],"PrefixListIds":[],"ToPort":443,"UserIdGroupPairs":[{"GroupId":"sg-04a97b1a6b3bdcb38","UserId":"835304779882"}]}],"OwnerId":"835304779882","GroupId":"sg-038993f5dffb85027","IpPermissionsEgress":[{"IpProtocol":"-1","IpRanges":[{"CidrIp":"0.0.0.0/0"}],"Ipv6Ranges":[],"PrefixListIds":[],"UserIdGroupPairs":[]}],"VpcId":"vpc-0eb60cd1e5650a5d9"}',
    ),
    null,
    2,
  ),
  Ingress: JSON.stringify(
    JSON.parse(
      '{"Description":"Allows production traffic to the app server","GroupName":"prod-app","IpPermissions":[{"FromPort":443,"IpProtocol":"tcp","IpRanges":[],"Ipv6Ranges":[],"PrefixListIds":[],"ToPort":443,"UserIdGroupPairs":[{"GroupId":"sg-04a97b1a6b3bdcb38","UserId":"835304779882"}]}],"OwnerId":"835304779882","GroupId":"sg-038993f5dffb85027","IpPermissionsEgress":[{"IpProtocol":"-1","IpRanges":[{"CidrIp":"0.0.0.0/0"}],"Ipv6Ranges":[],"PrefixListIds":[],"UserIdGroupPairs":[]}],"VpcId":"vpc-0eb60cd1e5650a5d9"}',
    ),
    null,
    2,
  ),
  "Security Group": JSON.stringify(
    JSON.parse(
      '{"Description":"Allows production traffic to the app server","GroupName":"prod-app","IpPermissions":[{"FromPort":443,"IpProtocol":"tcp","IpRanges":[],"Ipv6Ranges":[],"PrefixListIds":[],"ToPort":443,"UserIdGroupPairs":[{"GroupId":"sg-04a97b1a6b3bdcb38","UserId":"835304779882"}]}],"OwnerId":"835304779882","GroupId":"sg-038993f5dffb85027","IpPermissionsEgress":[{"IpProtocol":"-1","IpRanges":[{"CidrIp":"0.0.0.0/0"}],"Ipv6Ranges":[],"PrefixListIds":[],"UserIdGroupPairs":[]}],"VpcId":"vpc-0eb60cd1e5650a5d9"}',
    ),
    null,
    2,
  ),
  "Key Pair": JSON.stringify(
    JSON.parse(
      '{"KeyPairId":"key-03e2314b7a3002c8a","KeyFingerprint":"e0:28:a7:bf:07:62:3a:85:39:69:0c:c6:91:3d:4a:fa:84:be:e6:09","KeyName":"ssh_key_entity:1bb827f2-25d1-4fe1-aea7-8df481ddd1a3","KeyType":"rsa","Tags":[]}',
    ),
    null,
    2,
  ),
  "EC2 Instance": JSON.stringify(
    JSON.parse(
      '{"AmiLaunchIndex":0,"ImageId":"ami-0133ad8c5d900ddef","InstanceId":"i-0040189e75551cb18","InstanceType":"c5d.9xlarge","KeyName":"si_key","LaunchTime":"2022-02-07T20:09:57+00:00","Monitoring":{"State":"enabled"},"Placement":{"AvailabilityZone":"us-east-2b","GroupName":"","Tenancy":"default"},"PrivateDnsName":"ip-10-10-0-4.us-east-2.compute.internal","PrivateIpAddress":"10.10.0.4","ProductCodes":[],"PublicDnsName":"ec2-3-145-181-194.us-east-2.compute.amazonaws.com","PublicIpAddress":"3.145.181.194","State":{"Code":16,"Name":"running"},"StateTransitionReason":"","SubnetId":"subnet-060ae257a7a38dbf9","VpcId":"vpc-04eb54ec9ee5a1211","Architecture":"x86_64","BlockDeviceMappings":[{"DeviceName":"/dev/sda1","Ebs":{"AttachTime":"2022-02-07T20:09:58+00:00","DeleteOnTermination":true,"Status":"attached","VolumeId":"vol-0aad7ae0cfe126937"}}],"ClientToken":"","EbsOptimized":true,"EnaSupport":true,"Hypervisor":"xen","NetworkInterfaces":[{"Association":{"IpOwnerId":"amazon","PublicDnsName":"ec2-3-145-181-194.us-east-2.compute.amazonaws.com","PublicIp":"3.145.181.194"},"Attachment":{"AttachTime":"2022-02-07T20:09:57+00:00","AttachmentId":"eni-attach-00202e629619a3e19","DeleteOnTermination":true,"DeviceIndex":0,"Status":"attached","NetworkCardIndex":0},"Description":"Primary network interface","Groups":[{"GroupName":"outbound-only-ci","GroupId":"sg-0d15ffad7a65d0329"}],"Ipv6Addresses":[],"MacAddress":"06:f1:40:fa:1a:68","NetworkInterfaceId":"eni-0fe518271161abcdb","OwnerId":"835304779882","PrivateDnsName":"ip-10-10-0-4.us-east-2.compute.internal","PrivateIpAddress":"10.10.0.4","PrivateIpAddresses":[{"Association":{"IpOwnerId":"amazon","PublicDnsName":"ec2-3-145-181-194.us-east-2.compute.amazonaws.com","PublicIp":"3.145.181.194"},"Primary":true,"PrivateDnsName":"ip-10-10-0-4.us-east-2.compute.internal","PrivateIpAddress":"10.10.0.4"}],"SourceDestCheck":true,"Status":"in-use","SubnetId":"subnet-060ae257a7a38dbf9","VpcId":"vpc-04eb54ec9ee5a1211","InterfaceType":"interface"}],"RootDeviceName":"/dev/sda1","RootDeviceType":"ebs","SecurityGroups":[{"GroupName":"outbound-only-ci","GroupId":"sg-0d15ffad7a65d0329"}],"SourceDestCheck":true,"Tags":[{"Key":"env","Value":"ci"},{"Key":"createdBy","Value":"manual"},{"Key":"Name","Value":"ci-2"}],"VirtualizationType":"hvm","CpuOptions":{"CoreCount":18,"ThreadsPerCore":2},"CapacityReservationSpecification":{"CapacityReservationPreference":"open"},"HibernationOptions":{"Configured":false},"MetadataOptions":{"State":"applied","HttpTokens":"optional","HttpPutResponseHopLimit":1,"HttpEndpoint":"enabled","HttpProtocolIpv6":"disabled","InstanceMetadataTags":"disabled"},"EnclaveOptions":{"Enabled":false},"PlatformDetails":"Linux/UNIX","UsageOperation":"RunInstances","UsageOperationUpdateTime":"2022-02-07T20:09:57+00:00","PrivateDnsNameOptions":{"HostnameType":"ip-name","EnableResourceNameDnsARecord":true,"EnableResourceNameDnsAAAARecord":false}}',
    ),
    null,
    2,
  ),
  "Kubernetes Deployment":
    "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx-deployment\n  labels:\n    app: nginx\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: nginx\n  template:\n    metadata:\n      labels:\n        app: nginx\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:1.14.2\n        ports:\n        - containerPort: 80",
  Namespace: "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: whiskers",
} as { [key: string]: string };
