import { InstanceComponent as EC2InstanceComponent } from "./components/ec2/instance/InstanceComponent";
import { InstanceTypeComponent as EC2InstanceTypeComponent } from "./components/ec2/instance-type/InstanceTypeComponent";
import { SecurityGroupComponent as EC2SecurityGroupComponent } from "./components/ec2/security-group/SecurityGroupComponent";
import { VpcComponent as EC2VpcComponent } from "./components/ec2/vpc/VpcComponent";
import { NetworkingRuleComponent as EC2NetworkingRuleComponent } from "./components/ec2/networking-rule/NetworkingRuleComponent";
import { AmiComponent as EC2AmiComponent } from "./components/ec2/ami/AmiComponent";
import { KeyPairComponent as EC2KeyPairComponent } from "./components/ec2/keypair/KeyPairComponent";
import { NetworkingProtocolComponent as EC2NetworkingProtocolComponent } from "./components/ec2/networking-protocol/NetworkingProtocolComponent";
import { NetworkingCidrBlockComponent as EC2NetworkingCidrBlockComponent } from "./components/ec2/networking-cidr-block/NetworkingCidrBlockComponent";

export default {
  list: [
    new EC2InstanceComponent(),
    new EC2InstanceTypeComponent(),
    new EC2SecurityGroupComponent(),
    new EC2VpcComponent(),
    new EC2NetworkingRuleComponent(),
    new EC2AmiComponent(),
    new EC2KeyPairComponent(),
    new EC2NetworkingProtocolComponent(),
    new EC2NetworkingCidrBlockComponent(),
  ],
  get(name: string) {
    const comp = this.list.find(
      item => item.name.toUpperCase() === name.toUpperCase(),
    );

    if (!comp) throw new Error(`Rete component '${name}' not found`);
    return comp;
  },
};
