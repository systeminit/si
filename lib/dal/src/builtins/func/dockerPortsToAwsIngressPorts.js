function dockerPortsToAwsIngressPorts(input) {
  const dockerPortToObj = (entry) => {
    const [port, protocol] = entry.split('/');
    return {
      IpProtocol: protocol,
      ToPort: port,
      FromPort: port,
      CidrIp: '0.0.0.0/0',
    }
  };

  // NOTE(victor): This looks a bit weird, but it's because of the way the system passes in arguments on sockets
  // with multiple connections: If there's only one, it's an array of values, otherwise it's an array of arrays, one
  // for each connected port. There are examples for each payload on comments below the code
  const outputArray = input?.ExposedPorts?.flatMap((entry) => {
    if (Array.isArray(entry)) {
      return entry.map(dockerPortToObj)
    } else {
      return dockerPortToObj(entry)
    }
  }) ?? [];

  return outputArray;
}

// one connection:
// {"data":{"system":null,"kind":"standard","properties":{"ExposedPorts":["80/tcp"]},"resource":null},"parents":[]}
// Multiple connections:
// {"data":{"system":null,"kind":"standard","properties":{"ExposedPorts":[["80/tcp"],[]]},"resource":null},"parents":[]}
// One connection, array.length>1
// dockerPortsToAwsIngressPorts({"data":{"system":null,"kind":"standard","properties":{"ExposedPorts":["80/udp","81/tcp"]},"resource":null},"parents":[]})
