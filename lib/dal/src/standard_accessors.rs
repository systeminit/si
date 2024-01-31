#[macro_export]
macro_rules! standard_model_many_to_many {
    (
        lookup_fn: $lookup_fn:ident,
        associate_fn: $associate_fn:ident,
        disassociate_fn: $disassociate_fn:ident,
        disassociate_all_fn: $disassociate_all_fn:ident,
        table_name: $table_name:expr,
        left_table: $left_table:expr,
        left_id: $left_id:ident,
        right_table: $right_table:expr,
        right_id: $right_id:ident,
        which_table_is_this: "left",
        returns: $returns:ident,
        result: $result_type:ident $(,)?
    ) => {
        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $lookup_fn(
            &self,
            ctx: &$crate::DalContext,
        ) -> $result_type<Vec<$returns>> {
            let other: Option<&$right_id> = None;
            let r = $crate::standard_model::many_to_many(
                ctx,
                $table_name,
                $left_table,
                $right_table,
                Some(self.id()),
                other,
            )
            .await?;
            Ok(r)
        }

        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $associate_fn(
            &self,
            ctx: &$crate::DalContext,
            right_id: &$right_id,
        ) -> $result_type<()> {
            let _r = $crate::standard_model::associate_many_to_many(
                &ctx,
                $table_name,
                self.id(),
                right_id,
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($associate_fn)]),
                &Self::history_event_message(format!("associated {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "left_id": self.id(), "right_id": &right_id  }],
            )
            .await?;
            Ok(())
        }

        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $disassociate_fn(
            &self,
            ctx: &$crate::DalContext,
            right_id: &$right_id,
        ) -> $result_type<()> {
            let _r = $crate::standard_model::disassociate_many_to_many(
                ctx,
                $table_name,
                self.id(),
                right_id,
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($disassociate_fn)]),
                &Self::history_event_message(format!("disassociated {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "left_id": self.id(), "right_id": &right_id  }],
            )
            .await?;
            Ok(())
        }

        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $disassociate_all_fn(
            &self,
            ctx: &$crate::DalContext,
        ) -> $result_type<()> {
            $crate::standard_model::disassociate_all_many_to_many(
                ctx,
                $table_name,
                self.id(),
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($disassociate_fn)]),
                &Self::history_event_message(format!("disassociated all from {}", self.id)),
                &serde_json::json![{ "pk": self.pk, "left_id": self.id() }],
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
        result: $result_type:ident $(,)?
    ) => {
        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $lookup_fn(
            &self,
            ctx: &$crate::DalContext,
        ) -> $result_type<Vec<$returns>> {
            let other: Option<&$left_id> = None;
            let r = $crate::standard_model::many_to_many(
                ctx,
                $table_name,
                $left_table,
                $right_table,
                other,
                Some(self.id()),
            )
            .await?;
            Ok(r)
        }

        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $associate_fn(
            &self,
            ctx: &$crate::DalContext,
            left_id: &$left_id,
        ) -> $result_type<()> {
            let _r = $crate::standard_model::associate_many_to_many(
                ctx,
                $table_name,
                left_id,
                self.id(),
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($associate_fn)]),
                &Self::history_event_message(format!("associated {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "left_id": &left_id, "right_id": &self.id() }],
            )
            .await?;
            Ok(())
        }

        #[telemetry::tracing::instrument(skip_all, level = "trace")]
        pub async fn $disassociate_fn(
            &self,
            ctx: &$crate::DalContext,
            left_id: &$left_id,
        ) -> $result_type<()> {
            let _r = $crate::standard_model::disassociate_many_to_many(
                ctx,
                $table_name,
                left_id,
                self.id(),
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($disassociate_fn)]),
                &Self::history_event_message(format!("disassociated {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "right_id": self.id(), "left_id": &left_id }],
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
        result: $result_type:ident $(,)?
    ) => {
        #[telemetry::tracing::instrument(skip(ctx), level = "trace")]
        pub async fn $lookup_fn(&self, ctx: &$crate::DalContext) -> $result_type<Vec<$has_many>> {
            let r =
                $crate::standard_model::has_many(ctx, $table, $retrieve_table, &self.id()).await?;
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
        result: $result_type:ident $(,)?
    ) => {
        #[telemetry::tracing::instrument(skip(ctx), level = "trace")]
        pub async fn $lookup_fn(
            &self,
            ctx: &$crate::DalContext,
        ) -> $result_type<Option<$belongs_to>> {
            let r = $crate::standard_model::belongs_to(
                ctx,
                $table,
                $retrieve_table,
                &self.id(),
            )
            .await?;
            Ok(r)
        }

        #[telemetry::tracing::instrument(skip(ctx), level = "trace")]
        pub async fn $set_fn(
            &self,
            ctx: &$crate::DalContext,
            belongs_to_id: &$belongs_to_id,
        ) -> $result_type<()> {
            $crate::standard_model::set_belongs_to(
                ctx,
                $table,
                &self.id(),
                &belongs_to_id,
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($set_fn)]),
                &Self::history_event_message(format!("set {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "belongs_to_id": &belongs_to_id }],
            )
            .await?;
            Ok(())
        }

        #[telemetry::tracing::instrument(skip(ctx), level = "trace")]
        pub async fn $unset_fn(
            &self,
            ctx: &$crate::DalContext,
        ) -> $result_type<()> {
            $crate::standard_model::unset_belongs_to(
                ctx,
                $table,
                &self.id(),
            )
            .await?;
            let _history_event = $crate::HistoryEvent::new(
                ctx,
                &Self::history_event_label(vec![stringify!($unset_fn)]),
                &Self::history_event_message(format!("unset {}", stringify!($returns))),
                &serde_json::json![{ "pk": self.pk, "id": &self.id }],
            )
            .await?;
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! standard_model_accessor_ro {
    ($column:ident, $value_type:ty) => {
        pub fn $column(&self) -> &$value_type {
            &self.$column
        }
    };
}

#[macro_export]
macro_rules! standard_model_accessor {
    (@set_column $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        paste::paste! {
            #[telemetry::tracing::instrument(skip_all, level = "trace")]
            pub async fn [<set_ $column>](
                &mut self,
                ctx: &$crate::DalContext,
                value: impl Into<$value_type>,
            ) -> $result_type<()> {
                let value: $value_type = value.into();
                let updated_at = standard_model::update(
                    ctx,
                    Self::table_name(),
                    stringify!($column),
                    self.id(),
                    &value,
                    $hint,
                ).await?;
                let _history_event = $crate::HistoryEvent::new(
                    ctx,
                    &Self::history_event_label(vec!["updated"]),
                    &Self::history_event_message("updated"),
                    &serde_json::json![{
                        "pk": self.pk,
                        "field": stringify!($column),
                        "value": &value,
                    }],
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;

                Ok(())
            }
        }
    };

    (@set_column_copy $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        paste::paste! {
            #[telemetry::tracing::instrument(skip_all, level = "trace")]
            pub async fn [<set_ $column>](
                &mut self,
                ctx: &$crate::DalContext,
                value: impl Into<$value_type>,
            ) -> $result_type<()> {
                let value: $value_type = value.into();
                let updated_at = standard_model::update(
                    ctx,
                    Self::table_name(),
                    stringify!($column),
                    self.id(),
                    value,
                    $hint,
                ).await?;
                let _history_event = $crate::HistoryEvent::new(
                    ctx,
                    &Self::history_event_label(vec!["updated"]),
                    &Self::history_event_message("updated"),
                    &serde_json::json![{
                        "pk": self.pk,
                        "field": stringify!($column),
                        "value": &value,
                    }],
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;

                Ok(())
            }
        }
    };

    (@set_column_with_option $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        paste::paste! {
            #[telemetry::tracing::instrument(skip_all, level = "trace")]
            pub async fn [<set_ $column>](
                &mut self,
                ctx: &$crate::DalContext,
                value: Option<impl Into<$value_type>>,
            ) -> $result_type<()> {
                let value: Option<$value_type> = value.map(Into::into);
                let updated_at = standard_model::update(
                    ctx,
                    Self::table_name(),
                    stringify!($column),
                    self.id(),
                    &value,
                    $hint,
                ).await?;
                let _history_event = $crate::HistoryEvent::new(
                    ctx,
                    &Self::history_event_label(vec!["updated"]),
                    &Self::history_event_message("updated"),
                    &serde_json::json![{
                        "pk": self.pk,
                        "field": stringify!($column),
                        "value": &value,
                    }],
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    };

    (@set_column_as_ref $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        paste::paste! {
            #[telemetry::tracing::instrument(skip(ctx, value), level = "trace")]
            pub async fn [<set_ $column>](
                &mut self,
                ctx: &$crate::DalContext,
                value: impl Into<$value_type>,
            ) -> $result_type<()> {
                let value: $value_type = value.into();
                let updated_at = standard_model::update(
                    ctx,
                    Self::table_name(),
                    stringify!($column),
                    self.id(),
                    &value.as_ref(),
                    $hint,
                ).await?;
                let _history_event = $crate::HistoryEvent::new(
                    ctx,
                    &Self::history_event_label(vec!["updated"]),
                    &Self::history_event_message("updated"),
                    &serde_json::json![{
                        "pk": self.pk,
                        "field": stringify!($column),
                        "value": &value,
                    }],
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    };

    (@set_column_with_option_as_ref $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        paste::paste! {
            #[telemetry::tracing::instrument(skip_all, level = "trace")]
            pub async fn [<set_ $column>](
                &mut self,
                ctx: &$crate::DalContext,
                value: Option<impl Into<$value_type>>,
            ) -> $result_type<()> {
                let value: Option<$value_type> = value.map(Into::into);
                let updated_at = standard_model::update(
                    ctx,
                    Self::table_name(),
                    stringify!($column),
                    self.id(),
                    &value.as_ref().map(|v| v.as_ref()),
                    $hint,
                ).await?;
                let _history_event = $crate::HistoryEvent::new(
                    ctx,
                    &Self::history_event_label(vec!["updated"]),
                    &Self::history_event_message("updated"),
                    &serde_json::json![{
                        "pk": self.pk,
                        "field": stringify!($column),
                        "value": &value,
                    }],
                )
                .await?;
                self.timestamp.updated_at = updated_at;
                self.$column = value;
                Ok(())
            }
        }
    };

    (@get_column $column:ident, $value_type:ident $(,)?) => {
        pub fn $column(&self) -> &$value_type {
            &self.$column
        }
    };

    (@get_column_as_option $column:ident, $value_type:ident $(,)?) => {
        pub fn $column(&self) -> Option<&$value_type> {
            self.$column.as_ref()
        }
    };

    (@get_column_as_str $column:ident $(,)?) => {
        pub fn $column(&self) -> &str {
            &self.$column
        }
    };

    (@get_column_as_option_str $column:ident $(,)?) => {
        pub fn $column(&self) -> Option<&str> {
            self.$column.as_deref()
        }
    };

    (@get_column_copy $column:ident, $value_type:ident $(,)?) => {
        pub fn $column(&self) -> $value_type {
            self.$column
        }
    };

    (@copy_type $column:ident, $value_type:ident, $hint:ty, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_copy $column, $value_type);
        standard_model_accessor!(@set_column_copy $column, $value_type, $hint, $result_type);
    };

    ($column:ident, bool, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            bool,
            $crate::standard_model::TypeHint::Boolean,
            $result_type,
        );
    };

    ($column:ident, u8, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            u8,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, u16, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            u16,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, u32, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            u32,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, u64, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            u64,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, i8, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            i8,
            $crate::standard_model::TypeHint::Char,
            $result_type,
        );
    };

    ($column:ident, i16, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            i16,
            $crate::standard_model::TypeHint::SmallInt,
            $result_type,
        );
    };

    ($column:ident, i32, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            i32,
            $crate::standard_model::TypeHint::Integer,
            $result_type,
        );
    };

    ($column:ident, i64, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            i64,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, char, $result_type:ident $(,)?) => {
        standard_model_accessor!(@copy_type
            $column,
            char,
            $crate::standard_model::TypeHint::Char,
            $result_type,
        );
    };

    ($column:ident, Enum($value_type:ident), $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column $column, $value_type);
        standard_model_accessor!(@set_column_as_ref
            $column,
            $value_type,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, Pk($value_type:ident), $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_copy $column, $value_type);
        standard_model_accessor!(@set_column_copy
            $column,
            $value_type,
            $crate::standard_model::TypeHint::BpChar,
            $result_type,
        );
    };

    ($column:ident, String, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_str $column);
        standard_model_accessor!(@set_column
            $column,
            String,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, Option<String>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option_str $column);
        standard_model_accessor!(@set_column_with_option
            $column,
            String,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, $value_type:ident, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_str $column);
        standard_model_accessor!(@set_column
            $column,
            $value_type,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, Option<Enum($value_type:ident)>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, $value_type);
        standard_model_accessor!(@set_column_with_option_as_ref
            $column,
            $value_type,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, Option<Pk($value_type:ident)>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, $value_type);
        standard_model_accessor!(@set_column_with_option
            $column,
            $value_type,
            $crate::standard_model::TypeHint::BpChar,
            $result_type,
        );
    };

    ($column:ident, Option<bool>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, bool);
        standard_model_accessor!(@set_column_with_option
            $column,
            bool,
            $crate::standard_model::TypeHint::Boolean,
            $result_type,
        );
    };

    ($column:ident, Option<DateTimeUtc>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, DateTimeUtc);
        standard_model_accessor!(@set_column_with_option
            $column,
            DateTimeUtc,
            $crate::standard_model::TypeHint::TimestampWithTimeZone,
            $result_type,
        );
    };

    ($column:ident, Option<$value_type:ident>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, $value_type);
        standard_model_accessor!(@set_column_with_option
            $column,
            $value_type,
            $crate::standard_model::TypeHint::Text,
            $result_type,
        );
    };

    ($column:ident, OptionBigInt<$value_type:ident>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, $value_type);
        standard_model_accessor!(@set_column_with_option
            $column,
            $value_type,
            $crate::standard_model::TypeHint::BigInt,
            $result_type,
        );
    };

    ($column:ident, OptionJson<$value_type:ident>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column_as_option $column, $value_type);
        standard_model_accessor!(@set_column_with_option
            $column,
            $value_type,
            $crate::standard_model::TypeHint::JsonB,
            $result_type,
        );
    };

    ($column:ident, Json<$value_type:ident>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column $column, $value_type);
        standard_model_accessor!(@set_column
            $column,
            $value_type,
            $crate::standard_model::TypeHint::JsonB,
            $result_type,
        );
    };

    ($column:ident, PlainJson<$value_type:ident>, $result_type:ident $(,)?) => {
        standard_model_accessor!(@get_column $column, $value_type);
        standard_model_accessor!(@set_column
            $column,
            $value_type,
            $crate::standard_model::TypeHint::Json,
            $result_type,
        );
    };
}
