use crate::{ComponentId, DalContext};
use veritech_client::BeforeFunctionRequest;

pub async fn before_funcs_for_component(
    ctx: &DalContext,
    component_id: ComponentId,
) -> Vec<BeforeFunctionRequest> {
    // Function
    // component -> secretProp -> secretkind -> secrekindComp -> func
    // Arg
    // component -> secretProp -> value

    //Match func with arg

    vec![]
}
