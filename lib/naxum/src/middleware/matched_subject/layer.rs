use tower::Layer;

use super::{DefaultForSubject, MatchedSubject};

pub struct MatchedSubjectLayer<ForSubject = DefaultForSubject> {
    pub(crate) for_subject: ForSubject,
}

impl Default for MatchedSubjectLayer {
    fn default() -> Self {
        Self {
            for_subject: Default::default(),
        }
    }
}

impl MatchedSubjectLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<ForSubject> MatchedSubjectLayer<ForSubject> {
    pub fn for_subject<NewForSubject>(
        self,
        new_for_subject: NewForSubject,
    ) -> MatchedSubjectLayer<NewForSubject> {
        let Self { for_subject: _ } = self;
        MatchedSubjectLayer {
            for_subject: new_for_subject,
        }
    }
}

impl<S, ForSubject> Layer<S> for MatchedSubjectLayer<ForSubject>
where
    ForSubject: Clone,
{
    type Service = MatchedSubject<S, ForSubject>;

    fn layer(&self, inner: S) -> Self::Service {
        MatchedSubject {
            inner,
            for_subject: self.for_subject.clone(),
        }
    }
}
