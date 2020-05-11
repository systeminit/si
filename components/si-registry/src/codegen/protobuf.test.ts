import "../loader";
import { ProtobufFormatter } from "./protobuf";
import { registry } from "../registry";

test("protobuf imports", done => {
  const fmt = new ProtobufFormatter(
    registry.getObjectsForServiceName("kubernetes"),
  );
  const output = fmt.generateString();
  console.log(output);
  done();
});
