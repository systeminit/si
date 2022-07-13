use crate::{
    Component, ComponentError, ComponentId, DalContext, Job, JobFuture, QualificationsJob,
    StandardModel, SystemId, JobError,
};
use serde::{Deserialize, Serialize};

#[must_use]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
pub struct CodeGenerationJob {
    component_id: ComponentId,
    system_id: SystemId,
}

impl CodeGenerationJob {
    pub fn new(component_id: ComponentId, system_id: SystemId) -> Self {
        Self {
            component_id,
            system_id,
        }
    }
}

impl Job for CodeGenerationJob {
    fn prepare<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async {
            let component = Component::get_by_id(ctx, &self.component_id)
                .await?
                .ok_or(ComponentError::NotFound(self.component_id))
                .map_err(|err| JobError::Component(err.to_string()))?;
            component
                .prepare_code_generation(ctx, self.system_id)
                .await
                .map_err(|err| JobError::Component(err.to_string()))?;
            Ok(())
        })
    }

    fn run<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async {
            let component = Component::get_by_id(ctx, &self.component_id)
                .await?
                .ok_or(ComponentError::NotFound(self.component_id))
                .map_err(|err| JobError::Component(err.to_string()))?;

            component.generate_code(ctx, self.system_id).await
                .map_err(|err| JobError::Component(err.to_string()))?;

            ctx.enqueue_job(QualificationsJob::new(self.component_id, self.system_id))
                .await
                .map_err(|err| JobError::Component(err.to_string()))?;
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "CodeGenerationJob"
    }
}
