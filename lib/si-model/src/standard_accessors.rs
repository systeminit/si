#[macro_export]
macro_rules! standard_model_accessor {
    ($column:ident, $value_type:ident $(,)?) => {
        #[tracing::instrument]
        pub fn $column(&self) -> &str {
            &self.$column
        }

        paste::paste! {
            #[tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                history_actor: &crate::HistoryActor,
                value: impl Into<$value_type>,
            ) -> crate::StandardModelResult<()> {
                let value: $value_type = value.into();
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), &self.tenancy(), self.pk(), &value).await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec!["updated"]),
                    &history_actor,
                    &Self::history_event_message("updated"),
                    &serde_json::json![{ "pk": self.pk, "field": stringify!($column), "value": &value }],
                    &self.tenancy(),
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    };
    ($column:ident, Option<String> $(,)?) => {
        #[tracing::instrument]
        pub fn $column(&self) -> Option<&str> {
            self.$column.as_deref()
        }

        paste::paste! {
            #[tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                history_actor: &crate::HistoryActor,
                value: Option<String>,
            ) -> crate::StandardModelResult<()> {
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), &self.tenancy(), self.pk(), &value).await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec!["updated"]),
                    &history_actor,
                    &Self::history_event_message("updated"),
                    &serde_json::json![{ "pk": self.pk, "field": stringify!($column), "value": &value }],
                    &self.tenancy(),
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    };
    ($column:ident, Option<$value_type:ident> $(,)?) => {
        #[tracing::instrument]
        pub fn $column(&self) -> Option<&$value_type> {
            self.$column.as_ref()
        }

        paste::paste! {
            #[tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                history_actor: &crate::HistoryActor,
                value: Option<$value_type>,
            ) -> crate::StandardModelResult<()> {
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), self.tenancy(), self.pk(), &value).await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec!["updated"]),
                    &history_actor,
                    &Self::history_event_message("updated"),
                    &serde_json::json![{ "pk": self.pk, "field": stringify!($column), "value": &value }],
                    &self.tenancy(),
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    }
}
