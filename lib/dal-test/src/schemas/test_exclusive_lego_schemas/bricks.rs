use dal::prop::PropPath;
use dal::{BuiltinsResult, PropKind};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, FuncSpec, PropSpec, SocketSpec, SocketSpecData,
    SocketSpecKind,
};

use crate::schemas::schema_helpers::create_identity_func;

#[derive(Debug)]
pub(crate) struct LegoBricks {
    pub(crate) domain_name_prop: PropSpec,

    pub(crate) domain_one_prop: PropSpec,
    pub(crate) domain_two_prop: PropSpec,
    pub(crate) domain_three_prop: PropSpec,
    pub(crate) domain_four_prop: PropSpec,
    pub(crate) domain_five_prop: PropSpec,
    pub(crate) domain_six_prop: PropSpec,

    pub(crate) socket_one: SocketSpec,
    pub(crate) socket_two: SocketSpec,
    pub(crate) socket_three: SocketSpec,
    pub(crate) socket_four: SocketSpec,
    pub(crate) socket_five: SocketSpec,
    pub(crate) socket_six: SocketSpec,
}

impl LegoBricks {
    pub(crate) fn new_for_even() -> BuiltinsResult<Self> {
        let (identity_func_spec, domain_name_prop) = Self::setup()?;

        // Assemble all props.
        let domain_one_prop = PropSpec::builder()
            .name("one")
            .kind(PropKind::String)
            .build()?;
        let domain_two_prop = PropSpec::builder()
            .name("two")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("two")
                    .build()?,
            )
            .build()?;
        let domain_three_prop = PropSpec::builder()
            .name("three")
            .kind(PropKind::String)
            .build()?;
        let domain_four_prop = PropSpec::builder()
            .name("four")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("four")
                    .build()?,
            )
            .build()?;
        let domain_five_prop = PropSpec::builder()
            .name("five")
            .kind(PropKind::String)
            .build()?;
        let domain_six_prop = PropSpec::builder()
            .name("six")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("six")
                    .build()?,
            )
            .build()?;

        // Assemble all sockets.
        let socket_one = SocketSpec::builder()
            .name("one")
            .data(
                SocketSpecData::builder()
                    .name("one")
                    .connection_annotations(serde_json::to_string(&vec!["one"])?)
                    .kind(SocketSpecKind::Output)
                    .func_unique_id(&identity_func_spec.unique_id)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "one"]))
                    .build()?,
            )
            .build()?;
        let socket_two = SocketSpec::builder()
            .name("two")
            .data(
                SocketSpecData::builder()
                    .name("two")
                    .connection_annotations(serde_json::to_string(&vec!["two"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;
        let socket_three = SocketSpec::builder()
            .name("three")
            .data(
                SocketSpecData::builder()
                    .name("three")
                    .connection_annotations(serde_json::to_string(&vec!["three"])?)
                    .kind(SocketSpecKind::Output)
                    .func_unique_id(&identity_func_spec.unique_id)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "three"]))
                    .build()?,
            )
            .build()?;
        let socket_four = SocketSpec::builder()
            .name("four")
            .data(
                SocketSpecData::builder()
                    .name("four")
                    .connection_annotations(serde_json::to_string(&vec!["four"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;
        let socket_five = SocketSpec::builder()
            .name("five")
            .data(
                SocketSpecData::builder()
                    .name("five")
                    .connection_annotations(serde_json::to_string(&vec!["five"])?)
                    .kind(SocketSpecKind::Output)
                    .func_unique_id(&identity_func_spec.unique_id)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "five"]))
                    .build()?,
            )
            .build()?;
        let socket_six = SocketSpec::builder()
            .name("six")
            .data(
                SocketSpecData::builder()
                    .name("six")
                    .connection_annotations(serde_json::to_string(&vec!["six"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;

        Ok(Self {
            domain_name_prop,
            domain_one_prop,
            domain_two_prop,
            domain_three_prop,
            domain_four_prop,
            domain_five_prop,
            domain_six_prop,
            socket_one,
            socket_two,
            socket_three,
            socket_four,
            socket_five,
            socket_six,
        })
    }

    pub(crate) fn new_for_odd() -> BuiltinsResult<Self> {
        let (identity_func_spec, domain_name_prop) = Self::setup()?;

        // Assemble all props.
        let domain_one_prop = PropSpec::builder()
            .name("one")
            .kind(PropKind::String)
            .default_value(serde_json::json!("0"))
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("one")
                    .build()?,
            )
            .build()?;
        let domain_two_prop = PropSpec::builder()
            .name("two")
            .kind(PropKind::String)
            .build()?;
        let domain_three_prop = PropSpec::builder()
            .name("three")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("three")
                    .build()?,
            )
            .build()?;
        let domain_four_prop = PropSpec::builder()
            .name("four")
            .kind(PropKind::String)
            .build()?;
        let domain_five_prop = PropSpec::builder()
            .name("five")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::InputSocket)
                    .name("identity")
                    .socket_name("five")
                    .build()?,
            )
            .build()?;
        let domain_six_prop = PropSpec::builder()
            .name("six")
            .kind(PropKind::String)
            .build()?;

        // Assemble all sockets.
        let socket_one = SocketSpec::builder()
            .name("one")
            .data(
                SocketSpecData::builder()
                    .name("one")
                    .connection_annotations(serde_json::to_string(&vec!["one"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;
        let socket_two = SocketSpec::builder()
            .name("two")
            .data(
                SocketSpecData::builder()
                    .name("two")
                    .connection_annotations(serde_json::to_string(&vec!["two"])?)
                    .kind(SocketSpecKind::Output)
                    .func_unique_id(&identity_func_spec.unique_id)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "two"]))
                    .build()?,
            )
            .build()?;
        let socket_three = SocketSpec::builder()
            .name("three")
            .data(
                SocketSpecData::builder()
                    .name("three")
                    .connection_annotations(serde_json::to_string(&vec!["three"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;
        let socket_four = SocketSpec::builder()
            .name("four")
            .data(
                SocketSpecData::builder()
                    .name("four")
                    .connection_annotations(serde_json::to_string(&vec!["four"])?)
                    .kind(SocketSpecKind::Output)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "four"]))
                    .build()?,
            )
            .build()?;
        let socket_five = SocketSpec::builder()
            .name("five")
            .data(
                SocketSpecData::builder()
                    .name("five")
                    .connection_annotations(serde_json::to_string(&vec!["five"])?)
                    .kind(SocketSpecKind::Input)
                    .build()?,
            )
            .build()?;
        let socket_six = SocketSpec::builder()
            .name("six")
            .data(
                SocketSpecData::builder()
                    .name("six")
                    .connection_annotations(serde_json::to_string(&vec!["six"])?)
                    .kind(SocketSpecKind::Output)
                    .func_unique_id(&identity_func_spec.unique_id)
                    .build()?,
            )
            .input(
                AttrFuncInputSpec::builder()
                    .name("identity")
                    .kind(AttrFuncInputSpecKind::Prop)
                    .prop_path(PropPath::new(["root", "domain", "six"]))
                    .build()?,
            )
            .build()?;

        Ok(Self {
            domain_name_prop,
            domain_one_prop,
            domain_two_prop,
            domain_three_prop,
            domain_four_prop,
            domain_five_prop,
            domain_six_prop,
            socket_one,
            socket_two,
            socket_three,
            socket_four,
            socket_five,
            socket_six,
        })
    }

    fn setup() -> BuiltinsResult<(FuncSpec, PropSpec)> {
        let identity_func_spec = create_identity_func()?;

        let domain_name_prop = PropSpec::builder()
            .name("name")
            .kind(PropKind::String)
            .func_unique_id(&identity_func_spec.unique_id)
            .input(
                AttrFuncInputSpec::builder()
                    .kind(AttrFuncInputSpecKind::Prop)
                    .name("identity")
                    .prop_path(PropPath::new(["root", "si", "name"]))
                    .build()?,
            )
            .build()?;

        Ok((identity_func_spec, domain_name_prop))
    }
}
