use crate::{
    Component, ComponentError, ComponentId, DalContext, Job, JobError, JobFuture,
    QualificationPrototypeId, StandardModel, SystemId,
};
use serde::{Deserialize, Serialize};

#[must_use]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
pub struct QualificationsJob {
    component_id: ComponentId,
    system_id: SystemId,
}

impl QualificationsJob {
    pub fn new(component_id: ComponentId, system_id: SystemId) -> Self {
        Self {
            component_id,
            system_id,
        }
    }
}

impl Job for QualificationsJob {
    fn prepare<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async {
            let component = Component::get_by_id(ctx, &self.component_id)
                .await?
                .ok_or(ComponentError::NotFound(self.component_id))
                .map_err(|err| JobError::Component(err.to_string()))?;
            component
                .prepare_qualifications_check(ctx, self.system_id)
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

            component
                .check_qualifications(ctx, self.system_id)
                .await
                .map_err(|err| JobError::Component(err.to_string()))?;
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "QualificationsJob"
    }
}

#[must_use]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
pub struct QualificationJob {
    component_id: ComponentId,
    system_id: SystemId,
    prototype_id: QualificationPrototypeId,
}

impl QualificationJob {
    pub fn new(
        component_id: ComponentId,
        system_id: SystemId,
        prototype_id: QualificationPrototypeId,
    ) -> Self {
        Self {
            component_id,
            system_id,
            prototype_id,
        }
    }
}

impl Job for QualificationJob {
    fn prepare<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async {
            let component = Component::get_by_id(ctx, &self.component_id)
                .await?
                .ok_or(ComponentError::NotFound(self.component_id))
                .map_err(|err| JobError::Component(err.to_string()))?;
            component
                .prepare_qualification_check(ctx, self.system_id, self.prototype_id)
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
            component
                .check_qualification(ctx, self.system_id, self.prototype_id)
                .await
                .map_err(|err| JobError::Component(err.to_string()))?;
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "QualificationsJob"
    }
}
