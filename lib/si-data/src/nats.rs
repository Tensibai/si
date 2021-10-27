pub use async_nats::{Connection, Message, Subscription};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{info_span, instrument, Instrument, Span};

#[derive(Error, Debug)]
pub enum NatsTxnError {
    #[error("serde error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NatsTxnResult<T> = Result<T, NatsTxnError>;

#[derive(Clone, Debug)]
pub struct NatsConn {
    conn: Connection,
}

impl NatsConn {
    // TODO(fnichol): complete NatsConn instrumentation connection metadata
    #[instrument(name = "natsconn.new", skip(settings))]
    pub async fn new(settings: &si_settings::Nats) -> NatsTxnResult<Self> {
        let conn = async_nats::connect(&settings.url).await?;

        Ok(Self { conn })
    }

    // TODO(fnichol): add some form of `db.transaction` attribute
    #[instrument(name = "natsconn.transaction", skip(self))]
    pub fn transaction(&self) -> NatsTxn {
        NatsTxn::new(self.conn.clone(), Span::current())
    }

    #[instrument(name = "natsconn.subscribe", skip(self, subject))]
    pub async fn subscribe(&self, subject: &str) -> std::io::Result<Subscription> {
        self.conn.subscribe(subject).await
    }

    #[instrument(name = "natsconn.queue_subscribe", skip(self, subject, queue))]
    pub async fn queue_subscribe(
        &self,
        subject: &str,
        queue: &str,
    ) -> std::io::Result<Subscription> {
        self.conn.queue_subscribe(subject, queue).await
    }

    #[instrument(name = "natsconn.publish", skip(self))]
    pub async fn publish(&self, subject: &str, message: &str) -> std::io::Result<()> {
        self.conn.publish(subject, message).await
    }

    #[instrument(name = "natsconn.request_multi", skip(self))]
    pub async fn request_multi(
        &self,
        subject: &str,
        message: &str,
    ) -> std::io::Result<Subscription> {
        self.conn.request_multi(subject, message).await
    }
}

impl From<Connection> for NatsConn {
    fn from(conn: Connection) -> Self {
        NatsConn { conn }
    }
}

#[derive(Debug, Clone)]
pub struct NatsTxn {
    pub connection: Connection,
    object_list: Arc<Mutex<Vec<serde_json::Value>>>,
    tx_span: Span,
}

impl NatsTxn {
    fn new(connection: Connection, tx_span: Span) -> Self {
        NatsTxn {
            connection,
            object_list: Arc::new(Mutex::new(Vec::new())),
            tx_span,
        }
    }

    #[instrument(name = "natstxn.publish", skip(self, object))]
    pub async fn publish<T: Serialize + std::fmt::Debug>(&self, object: &T) -> NatsTxnResult<()> {
        let json: serde_json::Value = serde_json::to_value(object)?;
        let mut object_list = self.object_list.lock().await;
        object_list.push(json);
        Ok(())
    }

    #[instrument(name = "natstxn.delete", skip(self, object))]
    pub async fn delete<T: Serialize + std::fmt::Debug>(&self, object: &T) -> NatsTxnResult<()> {
        let json: serde_json::Value = serde_json::to_value(object)?;
        let mut object_list = self.object_list.lock().await;
        object_list.push(serde_json::json![{ "deleted": json }]);
        Ok(())
    }

    // TODO(fnichol): record a transaction attribute as committed
    #[instrument(name = "natstxn.commit", skip(self))]
    pub async fn commit(self) -> NatsTxnResult<()> {
        let mut object_list = self.object_list.lock().await;
        for model_json in object_list.iter_mut() {
            let mut model_body: serde_json::Value = model_json.clone();
            if model_json["deleted"].is_object() {
                model_body = model_json["deleted"].clone();
            }
            let mut subject_array: Vec<String> = Vec::new();
            if let Some(tenant_ids_values) = model_body["siStorable"]["tenantIds"].as_array() {
                for tenant_id_value in tenant_ids_values.iter() {
                    let tenant_id = String::from(tenant_id_value.as_str().unwrap());
                    subject_array.push(tenant_id);
                }
            } else {
                match model_body["siStorable"]["billingAccountId"].as_str() {
                    Some(billing_account_id) => subject_array.push(billing_account_id.into()),
                    None => return Ok(()),
                }
            }
            if !subject_array.is_empty() {
                let subject: String = subject_array.join(".");
                self.connection
                    .publish(&subject, model_json.to_string())
                    .instrument(info_span!(
                        "publish",
                        code.namespace = "async-nats::Connection"
                    ))
                    .await?;
            } else {
                dbg!(
                    "tried to publish a model that has no tenancy; model_json={:?}",
                    model_json
                );
            }
        }
        Ok(())
    }
}