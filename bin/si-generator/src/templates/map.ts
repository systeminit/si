export const partial = `
.setName("<%= it.prop.name %>")
.setKind("map")
.setEntry(
<%~ include("@renderPropPartial", { prop: it.prop.entry, omitVariable: true }) %>
)
`;
