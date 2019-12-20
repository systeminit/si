#!/usr/bin/perl
print <<EOH;
import { Port, PortComponent } from "@/datalayer/component/port";
import { Integration } from "@/datalayer/integration";

export async function getPortData(): Promise<PortComponent[]> {
  const globalIntegration = await Integration.getByName("Global");
  const data = [
EOH
while (<>) { 
  my $line = $_;

  if ($line =~ /^(\w+?)\s+(\d+)\/(.+)$/) {
    my $service_name = $1;
    my $port = $2;
    my $protocol = $3;
    my $uuid = `uuidgen`;
    chomp($uuid);

    print <<EOF;
    Port.New({
      id: "$uuid",
      name: "$service_name $protocol",
      description: "IANA $service_name $protocol",
      rawDataJson: "{}",
      supportedActions: [],
      integrationId: globalIntegration.fqId,
      serviceName: "$service_name",
      number: $port,
      protocol: "$protocol",
    }),
EOF
  }
}
print <<EOH;
  ];
  return data;
}
EOH

