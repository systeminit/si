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
      '{ "AmiLaunchIndex": 0, "ImageId": "ami-0bde60638be9bb870", "InstanceId": "i-0a22fff028cb843cc", "InstanceType": "t3.micro", "KeyName": "si_key", "LaunchTime": "2022-10-20T02:56:37+00:00", "Monitoring": { "State": "disabled" }, "Placement": { "AvailabilityZone": "us-east-2a", "GroupName": "", "Tenancy": "default" }, "PrivateDnsName": "ip-10-1-1-60.us-east-2.compute.internal", "PrivateIpAddress": "10.1.1.60", "ProductCodes": [], "PublicDnsName": "", "PublicIpAddress": "3.144.118.6", "State": { "Code": 16, "Name": "running" }, "StateTransitionReason": "", "SubnetId": "subnet-07d580fee7a806230", "VpcId": "vpc-0eb60cd1e5650a5d9", "Architecture": "x86_64", "BlockDeviceMappings": [ { "DeviceName": "/dev/xvda", "Ebs": { "AttachTime": "2022-10-20T02:56:38+00:00", "DeleteOnTermination": true, "Status": "attached", "VolumeId": "vol-04ce6a1c94e4de564" } } ], "ClientToken": "71BB2462-D92E-40DC-A1A1-39CF5F2D2AF5", "EbsOptimized": false, "EnaSupport": true, "Hypervisor": "xen", "NetworkInterfaces": [ { "Association": { "IpOwnerId": "amazon", "PublicDnsName": "", "PublicIp": "3.144.118.6" }, "Attachment": { "AttachTime": "2022-10-20T02:56:37+00:00", "AttachmentId": "eni-attach-06577325aa1fe3dc6", "DeleteOnTermination": true, "DeviceIndex": 0, "Status": "attached", "NetworkCardIndex": 0 }, "Description": "", "Groups": [ { "GroupName": "allow_http", "GroupId": "sg-04520152ef738869b" } ], "Ipv6Addresses": [], "MacAddress": "02:d8:bc:ac:cc:46", "NetworkInterfaceId": "eni-0ed823869ca596464", "OwnerId": "835304779882", "PrivateIpAddress": "10.1.1.60", "PrivateIpAddresses": [ { "Association": { "IpOwnerId": "amazon", "PublicDnsName": "", "PublicIp": "3.144.118.6" }, "Primary": true, "PrivateIpAddress": "10.1.1.60" } ], "SourceDestCheck": true, "Status": "in-use", "SubnetId": "subnet-07d580fee7a806230", "VpcId": "vpc-0eb60cd1e5650a5d9", "InterfaceType": "interface" } ], "RootDeviceName": "/dev/xvda", "RootDeviceType": "ebs", "SecurityGroups": [ { "GroupName": "allow_http", "GroupId": "sg-04520152ef738869b" } ], "SourceDestCheck": true, "Tags": [ { "Key": "Name", "Value": "whiskers" }, ], "VirtualizationType": "hvm", "CpuOptions": { "CoreCount": 1, "ThreadsPerCore": 2 }, "CapacityReservationSpecification": { "CapacityReservationPreference": "open" }, "HibernationOptions": { "Configured": false }, "MetadataOptions": { "State": "applied", "HttpTokens": "optional", "HttpPutResponseHopLimit": 1, "HttpEndpoint": "enabled", "HttpProtocolIpv6": "disabled", "InstanceMetadataTags": "disabled" }, "EnclaveOptions": { "Enabled": false }, "PlatformDetails": "Linux/UNIX", "UsageOperation": "RunInstances", "UsageOperationUpdateTime": "2022-10-20T02:56:37+00:00", "PrivateDnsNameOptions": { "HostnameType": "ip-name", "EnableResourceNameDnsARecord": false, "EnableResourceNameDnsAAAARecord": false } }',
    ),
    null,
    2,
  ),
  "Kubernetes Deployment":
    "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx-deployment\n  labels:\n    app: nginx\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: nginx\n  template:\n    metadata:\n      labels:\n        app: nginx\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:1.14.2\n        ports:\n        - containerPort: 80",
  Namespace: "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: whiskers",
} as { [key: string]: string };
