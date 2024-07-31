use axum::extract::FromRef;
use typed_builder::TypedBuilder;

use crate::mqtta::MqttHandle;

#[derive(Clone, FromRef, TypedBuilder)]
pub(crate) struct AppState {
    mqtt: MqttHandle,
}
