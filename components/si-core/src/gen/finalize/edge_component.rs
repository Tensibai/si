// Auto-generated code!
// No touchy!

use si_cea::FinalizeBuilder;
use std::convert::TryInto;

struct EdgeComponentFinalizerBuilder {
    finalize_key: Option<si_cea::FinalizeKey>,
}

impl si_cea::FinalizeBuilder for EdgeComponentFinalizerBuilder {
    type Finalizeable = EdgeComponentFinalizer;

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
        "edge_component"
    }
}

pub struct EdgeComponentFinalizer {
    finalize_key: si_cea::FinalizeKey,
}

impl EdgeComponentFinalizer {
    fn new(finalize_key: si_cea::FinalizeKey) -> Self {
        Self { finalize_key }
    }
}

#[async_trait::async_trait]
impl si_cea::Finalize for EdgeComponentFinalizer {
    async fn finalize(
        &self,
        db: &si_data::Db,
        message: si_agent::WireMessage,
    ) -> si_cea::CeaResult<()> {
        let (_header, _qos, _response_header, object) = {
            let msg: si_agent::Message<crate::protobuf::EdgeComponent> = message.try_into()?;
            msg.into_parts()
        };

        object.finalize(db).await?;

        Ok(())
    }

    fn finalize_key(&self) -> si_cea::FinalizeKey {
        self.finalize_key.clone()
    }
}

impl si_cea::Finalizeable for EdgeComponentFinalizer {}

pub fn finalizer() -> si_cea::CeaResult<EdgeComponentFinalizer> {
    let mut builder = EdgeComponentFinalizerBuilder::new();
    builder.finalize_key(si_cea::FinalizeKey::new(builder.object_type()));

    builder.build()
}
