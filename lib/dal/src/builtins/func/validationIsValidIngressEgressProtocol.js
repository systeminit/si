const INGRESS_EGRESS_PROTOS = ["tcp", "udp", "icmp"];
const PROTO_STRING = INGRESS_EGRESS_PROTOS.map(proto => `'${proto}'`).join(", ");

function isValidIngressEgressProtocol(input) {
    const proto = input.value;
    const valid = INGRESS_EGRESS_PROTOS.indexOf(proto) !== -1;
    return {
        valid,
        message: valid ? undefined : `'${proto}' must be one of ${PROTO_STRING}`
    };
}