#[macro_export]
macro_rules! standard_model_many_to_many {
    (
        lookup_fn: $lookup_fn:ident,
        associate_fn: $associate_fn:ident,
        disassociate_fn: $disassociate_fn:ident,
        table_name: $table_name:expr,
        left_table: $left_table:expr,
        left_id: $left_id:ident,
        right_table: $right_table:expr,
        right_id: $right_id:ident,
        which_table_is_this: "left",
        returns: $returns:ident,
        result: $result_type:ident $(,)? ) => {

            #[telemetry::tracing::instrument(skip(txn))]
            pub async fn $lookup_fn(&self, txn: &si_data::PgTxn<'_>) -> $result_type<Vec<$returns>> {
                let other: Option<&$right_id> = None;
                let r = crate::standard_model::many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    $left_table,
                    $right_table,
                    Some(self.id()),
                    other,
                )
                .await?;
                Ok(r)
            }

            paste::paste! {
                #[telemetry::tracing::instrument(skip(txn))]
                pub async fn [<$lookup_fn _with_tenancy>](&self, txn: &si_data::PgTxn<'_>, tenancy: &crate::Tenancy) -> $result_type<Vec<$returns>> {
                    let other: Option<&$right_id> = None;
                    let r = crate::standard_model::many_to_many(
                        &txn,
                        $table_name,
                        tenancy,
                        &self.visibility(),
                        $left_table,
                        $right_table,
                        Some(self.id()),
                        other,
                    )
                    .await?;
                    Ok(r)
                }
            }

            #[telemetry::tracing::instrument(skip(txn, nats))]
            pub async fn $associate_fn(&self, txn: &si_data::PgTxn<'_>, nats: &si_data::NatsTxn, history_actor: &crate::HistoryActor, right_id: &$right_id) -> $result_type<()> {
                let _r = crate::standard_model::associate_many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    self.id(),
                    right_id,
                )
                .await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec![stringify!($associate_fn)]),
                    &history_actor,
                    &Self::history_event_message(format!("associated {}", stringify!($returns))),
                    &serde_json::json![{ "pk": self.pk, "left_id": self.id(), "right_id": &right_id  }],
                    &self.tenancy(),
                )
                .await?;
                Ok(())
            }

            #[telemetry::tracing::instrument(skip(txn, nats))]
            pub async fn $disassociate_fn(&self, txn: &si_data::PgTxn<'_>, nats: &si_data::NatsTxn, history_actor: &crate::HistoryActor, right_id: &$right_id) -> $result_type<()> {
                let _r = crate::standard_model::disassociate_many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    self.id(),
                    right_id,
                )
                .await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec![stringify!($disassociate_fn)]),
                    &history_actor,
                    &Self::history_event_message(format!("disassociated {}", stringify!($returns))),
                    &serde_json::json![{ "pk": self.pk, "left_id": self.id(), "right_id": &right_id  }],
                    &self.tenancy(),
                )
                .await?;
                Ok(())
            }
    };
    (
        lookup_fn: $lookup_fn:ident,
        associate_fn: $associate_fn:ident,
        disassociate_fn: $disassociate_fn:ident,
        table_name: $table_name:expr,
        left_table: $left_table:expr,
        left_id: $left_id:ident,
        right_table: $right_table:expr,
        right_id: $right_id:ident,
        which_table_is_this: "right",
        returns: $returns:ident,
        result: $result_type:ident $(,)? ) => {

            #[telemetry::tracing::instrument(skip(txn, nats))]
            pub async fn $lookup_fn(&self, txn: &si_data::PgTxn<'_>) -> $result_type<Vec<$returns>> {
                let other: Option<&$left_id> = None;
                let r = crate::standard_model::many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    $left_table,
                    $right_table,
                    other,
                    Some(self.id()),
                )
                .await?;
                Ok(r)
            }

            #[telemetry::tracing::instrument(skip(txn, nats))]
            pub async fn $associate_fn(&self, txn: &si_data::PgTxn<'_>, nats: &si_data::NatsTxn, history_actor: &crate::HistoryActor, left_id: &$left_id) -> $result_type<()> {
                let _r = crate::standard_model::associate_many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    left_id,
                    self.id(),
                )
                .await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec![stringify!($associate_fn)]),
                    &history_actor,
                    &Self::history_event_message(format!("associated {}", stringify!($returns))),
                    &serde_json::json![{ "pk": self.pk, "left_id": &left_id, "right_id": &self.id() }],
                    &self.tenancy(),
                )
                .await?;
                Ok(())
            }

            #[telemetry::tracing::instrument(skip(txn, nats))]
            pub async fn $disassociate_fn(&self, txn: &si_data::PgTxn<'_>, nats: &si_data::NatsTxn, history_actor: &crate::HistoryActor, left_id: &$left_id) -> $result_type<()> {
                let _r = crate::standard_model::disassociate_many_to_many(
                    &txn,
                    $table_name,
                    &self.tenancy(),
                    &self.visibility(),
                    left_id,
                    self.id(),
                )
                .await?;
                let _history_event = crate::HistoryEvent::new(
                    &txn,
                    &nats,
                    &Self::history_event_label(vec![stringify!($disassociate_fn)]),
                    &history_actor,
                    &Self::history_event_message(format!("disassociated {}", stringify!($returns))),
                    &serde_json::json![{ "pk": self.pk, "right_id": self.id(), "left_id": &left_id }],
                    &self.tenancy(),
                )
                .await?;
                Ok(())
            }
    };

}

#[macro_export]
macro_rules! standard_model_has_many {
    (
        lookup_fn: $lookup_fn:ident,
        table: $table:expr,
        model_table: $retrieve_table:expr,
        returns: $has_many:ident,
        result: $result_type:ident $(,)? ) => {
        #[telemetry::tracing::instrument(skip(txn))]
        pub async fn $lookup_fn(&self, txn: &si_data::PgTxn<'_>) -> $result_type<Vec<$has_many>> {
            let r = crate::standard_model::has_many(
                &txn,
                $table,
                &self.tenancy(),
                &self.visibility(),
                $retrieve_table,
                &self.id(),
            )
            .await?;
            Ok(r)
        }
    };
}

#[macro_export]
macro_rules! standard_model_belongs_to {
    (
        lookup_fn: $lookup_fn:ident,
        set_fn: $set_fn:ident,
        unset_fn: $unset_fn:ident,
        table: $table:expr,
        model_table: $retrieve_table:expr,
        belongs_to_id: $belongs_to_id:ident,
        returns: $belongs_to:ident,
        result: $result_type:ident $(,)? ) => {

        #[telemetry::tracing::instrument(skip(txn))]
        pub async fn $lookup_fn(
            &self,
            txn: &si_data::PgTxn<'_>,
        ) -> $result_type<Option<$belongs_to>> {
            let r = crate::standard_model::belongs_to(
                &txn,
                $table,
                &self.tenancy(),
                &self.visibility(),
                $retrieve_table,
                &self.id(),
            )
            .await?;
            Ok(r)
        }

        paste::paste! {
            #[telemetry::tracing::instrument(skip(txn))]
            pub async fn [<$lookup_fn _with_tenancy>](
                &self,
                txn: &si_data::PgTxn<'_>,
                tenancy: &crate::Tenancy,
            ) -> $result_type<Option<$belongs_to>> {
                let r = crate::standard_model::belongs_to(
                    &txn,
                    $table,
                    &self.tenancy(),
                    &self.visibility(),
                    $retrieve_table,
                    &self.id(),
                )
                .await?;
                Ok(r)
            }
        }

        #[telemetry::tracing::instrument(skip(txn))]
        pub async fn $set_fn(
            &self,
            txn: &si_data::PgTxn<'_>,
            nats: &si_data::NatsTxn,
            history_actor: &crate::HistoryActor,
            belongs_to_id: &$belongs_to_id,
        ) -> $result_type<()> {
            crate::standard_model::set_belongs_to(
                &txn,
                $table,
                &self.tenancy(),
                &self.visibility(),
                &self.id(),
                &belongs_to_id,
            )
            .await?;
            let _history_event = crate::HistoryEvent::new(
                &txn,
                &nats,
                &Self::history_event_label(vec![stringify!($set_fn)]),
                &history_actor,
                &Self::history_event_message(format!("set {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "belongs_to_id": &belongs_to_id }],
                &self.tenancy(),
            )
            .await?;
            Ok(())
        }

        #[telemetry::tracing::instrument(skip(txn))]
        pub async fn $unset_fn(
            &self,
            txn: &si_data::PgTxn<'_>,
            nats: &si_data::NatsTxn,
            history_actor: &crate::HistoryActor,
        ) -> $result_type<()> {
            crate::standard_model::unset_belongs_to(
                &txn,
                $table,
                &self.tenancy(),
                &self.visibility(),
                &self.id(),
            )
            .await?;
            let _history_event = crate::HistoryEvent::new(
                &txn,
                &nats,
                &Self::history_event_label(vec![stringify!($unset_fn)]),
                &history_actor,
                &Self::history_event_message(format!("unset {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "id": &self.id }],
                &self.tenancy(),
            )
            .await?;
            Ok(())
        }

    };
}

#[macro_export]
macro_rules! standard_model_accessor_ro {
    ($column:ident, $value_type:ident) => {
        #[telemetry::tracing::instrument]
        pub fn $column(&self) -> &$value_type {
            &self.$column
        }
    };
}

#[macro_export]
macro_rules! standard_model_accessor {
    ($column:ident, $value_type:ident, $result_type:ident $(,)?) => {
        #[telemetry::tracing::instrument]
        pub fn $column(&self) -> &str {
            &self.$column
        }

        paste::paste! {
            #[telemetry::tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                visibility: &crate::Visibility,
                history_actor: &crate::HistoryActor,
                value: impl Into<$value_type>,
            ) -> $result_type<()> {
                let value: $value_type = value.into();
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), &self.tenancy(), &visibility, self.id(), &value).await?;
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
    ($column:ident, Enum($value_type:ident), $result_type:ident $(,)?) => {
        #[telemetry::tracing::instrument]
        pub fn $column(&self) -> &$value_type {
            &self.$column
        }

        paste::paste! {
            #[telemetry::tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                visibility: &crate::Visibility,
                history_actor: &crate::HistoryActor,
                value: impl Into<$value_type>,
            ) -> $result_type<()> {
                let value: $value_type = value.into();
                let value_string = value.to_string();
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), &self.tenancy(), &visibility, self.id(), &value_string).await?;
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

    ($column:ident, Option<String>, $result_type:ident $(,)?) => {
        #[telemetry::tracing::instrument]
        pub fn $column(&self) -> Option<&str> {
            self.$column.as_deref()
        }

        paste::paste! {
            #[telemetry::tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                visibility: &crate::Visibility,
                history_actor: &crate::HistoryActor,
                value: Option<String>,
            ) -> $result_type<()> {
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), &self.tenancy(), &visibility, self.id(), &value).await?;
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
    ($column:ident, Option<$value_type:ident>, $result_type:ident $(,)?) => {
        #[telemetry::tracing::instrument]
        pub fn $column(&self) -> Option<&$value_type> {
            self.$column.as_ref()
        }

        paste::paste! {
            #[telemetry::tracing::instrument(skip(txn, nats, value))]
            pub async fn [<set_ $column>](
                &mut self,
                txn: &si_data::PgTxn<'_>,
                nats: &si_data::NatsTxn,
                visibility: &crate::Visibility,
                history_actor: &crate::HistoryActor,
                value: Option<$value_type>,
            ) -> $result_type<()> {
                let updated_at =
                        standard_model::update(&txn, Self::table_name(), stringify!($column), self.tenancy(), &visibility, self.id(), &value).await?;
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
