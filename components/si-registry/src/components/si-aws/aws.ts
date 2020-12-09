import { PropSelect } from "../../components/prelude";
import { ComponentAndEntityObject } from "../../systemComponent";
import { registry } from "../../registry";

let aws = {
  typeName: "aws",
  displayTypeName: "AWS",
  siPathName: "si-core",
  serviceName: "core",
  options(c: ComponentAndEntityObject) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "aws",
      uiMenuDisplayName: "region",
    };
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    c.entity.properties.addSelect({
      name: "region",
      label: "region",
      options(p: PropSelect) {
        p.required = true;
        p.options = [
          { key: "US East (N. Virginia) us-east-1", value: "us-east-1" },
          { key: "US East (Ohio) us-east-2", value: "us-east-2" },
          { key: "US West (N. California) us-west-1", value: "us-west-1" },
          { key: "US West (Oregon) us-west-2", value: "us-west-2" },
        ];
      },
    });
  },
};

export { aws };

registry.componentAndEntity(aws);
