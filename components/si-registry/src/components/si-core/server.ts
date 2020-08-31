import { PropAction, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";
import _ from "lodash";
import { CalculateConfiguresReply } from "@/veritech/intelligence";

registry.componentAndEntity({
  typeName: "server",
  displayTypeName: "Server",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    c.properties.addSelect({
      name: "operatingSystem",
      label: "operatingSystem",
      options(p: PropSelect) {
        p.options = [
          {
            key: "Ubuntu 20.04 LTS",
            value: "Ubuntu 20.04 LTS",
          },
          {
            key: "Ubuntu 20.04.1 LTS",
            value: "Ubuntu 20.04.1 LTS",
          },
          {
            key: "Ubuntu 18.04.5 LTS",
            value: "Ubuntu 18.04.5 LTS",
          },
          {
            key: "Ubuntu 18.04.4 LTS",
            value: "Ubuntu 18.04.4 LTS",
          },
          {
            key: "Ubuntu 18.04.3 LTS",
            value: "Ubuntu 18.04.3 LTS",
          },
          {
            key: "Ubuntu 18.04.2 LTS",
            value: "Ubuntu 18.04.2 LTS",
          },
          {
            key: "Ubuntu 18.04.1 LTS",
            value: "Ubuntu 18.04.1 LTS",
          },
          {
            key: "Ubuntu 18.04 LTS",
            value: "Ubuntu 18.04 LTS",
          },
          {
            key: "Red Hat Enterprise Linux 8",
            value: "RHEL 8",
          },
          {
            key: "Red Hat Enterprise Linux 8.2",
            value: "RHEL 8.2",
          },
          {
            key: "Red Hat Enterprise Linux 8.1",
            value: "RHEL 8.1",
          },
          {
            key: "Red Hat Enterprise Linux 7",
            value: "RHEL 7",
          },
          {
            key: "Red Hat Enterprise Linux 7.8",
            value: "RHEL 7.8",
          },
          {
            key: "Red Hat Enterprise Linux 7.7",
            value: "RHEL 7.7",
          },
          {
            key: "Red Hat Enterprise Linux 7.6",
            value: "RHEL 7.6",
          },
          {
            key: "Red Hat Enterprise Linux 7.5",
            value: "RHEL 7.5",
          },
          {
            key: "Red Hat Enterprise Linux 7.4",
            value: "RHEL 7.4",
          },
          {
            key: "Red Hat Enterprise Linux 7.3",
            value: "RHEL 7.3",
          },
          {
            key: "Red Hat Enterprise Linux 7.2",
            value: "RHEL 7.2",
          },
          {
            key: "Red Hat Enterprise Linux 7.1",
            value: "RHEL 7.1",
          },
        ];
      },
    });

    c.properties.addSelect({
      name: "cpu",
      label: "cpu",
      options(p: PropSelect) {
        p.options = [
          {
            key: "Intel x86_64",
            value: "Intel x86_64",
          },
          // AWS M4
          {
            key: "2.3 GHz Intel Xeon E5-2686 v4 (Broadwell)",
            value: "2.3 GHz Intel Xeon E5-2686 v4 (Broadwell)",
          },
          {
            key: "2.4 GHz Intel Xeon E5-2676 v3 (Haswell)",
            value: "2.4 GHz Intel Xeon E5-2676 v3 (Haswell)",
          },
          // AWS M5
          {
            key: "3.1 GHz Intel Xeon Platinum 8175M (AVX-512)",
            value: "3.1 GHz Intel Xeon Platinum 8175M (AVX-512)",
          },
        ];
      },
    });
    c.properties.addSelect({
      name: "memory",
      label: "memory",
      options(p: PropSelect) {
        p.options = [
          {
            key: "512MiB",
            value: "512MiB",
          },
          { key: "1GiB", value: "1GiB" },
          { key: "2GiB", value: "2GiB" },
          { key: "3GiB", value: "3GiB" },
          { key: "4GiB", value: "4GiB" },
          {
            key: "8GiB",
            value: "8GiB",
          },
          {
            key: "16GiB",
            value: "16GiB",
          },
          {
            key: "32GiB",
            value: "32GiB",
          },
        ];
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "start",
      label: "Start",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
    c.entity.methods.addAction({
      name: "stop",
      label: "Stop",
      options(p: PropAction) {
        p.mutation = true;
      },
    });

    // Entity Intelligence
    c.entity.intelligence.calculateConfigures = (
      entity,
      configures,
      systems,
    ): CalculateConfiguresReply => {
      const allSystems = _.map(systems, s => s.id);
      const keep = _.map(configures, c => {
        return { id: c.id, systems: allSystems };
      });
      const result: CalculateConfiguresReply = {
        keep,
      };
      if (!_.find(configures, ["objectType", "ubuntu"])) {
        const create: CalculateConfiguresReply["create"] = [
          {
            objectType: "ubuntu",
            name: `${entity.name} OS`,
            systems: allSystems,
          },
        ];
        result.create = create;
      }
      console.log("please do", { result });
      return result;
    };
  },
});
