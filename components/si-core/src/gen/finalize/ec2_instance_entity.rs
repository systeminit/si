// Auto-generated code!
// No touchy!

use si_cea::FinalizeBuilder;
use std::convert::TryInto;

struct Ec2InstanceEntityFinalizerBuilder {
    finalize_key: Option<si_cea::FinalizeKey>,
}

impl si_cea::FinalizeBuilder for Ec2InstanceEntityFinalizerBuilder {
    type Finalizeable = Ec2InstanceEntityFinalizer;

    fn new() -> Self {
        Self { finalize_key: None }
    }

    fn finalize_key(&mut self, finalize_key: si_cea::FinalizeKey) -> &mut Self {
        self.finalize_key = Some(finalize_key);
        self
    }

    fn build(self) -> si_cea::CeaResult<Self::Finalizeable> {
        let finalize_key = self
            .finalize_key
            .ok_or(si_cea::CeaError::MissingFinalizeKey)?;

        Ok(Self::Finalizeable::new(finalize_key))
    }

    fn object_type(&self) -> &'static str {
        "ec2_instance_entity"
    }
}

pub struct Ec2InstanceEntityFinalizer {
    finalize_key: si_cea::FinalizeKey,
}

impl Ec2InstanceEntityFinalizer {
    fn new(finalize_key: si_cea::FinalizeKey) -> Self {
        Self { finalize_key }
    }
}

#[async_trait::async_trait]
impl si_cea::Finalize for Ec2InstanceEntityFinalizer {
    async fn finalize(
        &self,
        db: &si_data::Db,
        message: si_agent::WireMessage,
    ) -> si_cea::CeaResult<()> {
        let (_header, _qos, _response_header, object) = {
            let msg: si_agent::Message<crate::protobuf::Ec2InstanceEntity> = message.try_into()?;
            msg.into_parts()
        };

        object.finalize(db).await?;

        Ok(())
    }

    fn finalize_key(&self) -> si_cea::FinalizeKey {
        self.finalize_key.clone()
    }
}

impl si_cea::Finalizeable for Ec2InstanceEntityFinalizer {}

pub fn finalizer() -> si_cea::CeaResult<Ec2InstanceEntityFinalizer> {
    let mut builder = Ec2InstanceEntityFinalizerBuilder::new();
    builder.finalize_key(si_cea::FinalizeKey::new(builder.object_type()));

    builder.build()
}
