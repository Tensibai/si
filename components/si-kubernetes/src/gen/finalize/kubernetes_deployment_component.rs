// Auto-generated code!
// No touchy!

use si_cea::FinalizeBuilder;
use std::convert::TryInto;

struct KubernetesDeploymentComponentFinalizerBuilder {
    finalize_key: Option<si_cea::FinalizeKey>,
}

impl si_cea::FinalizeBuilder for KubernetesDeploymentComponentFinalizerBuilder {
    type Finalizeable = KubernetesDeploymentComponentFinalizer;

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
        "kubernetes_deployment_component"
    }
}

pub struct KubernetesDeploymentComponentFinalizer {
    finalize_key: si_cea::FinalizeKey,
}

impl KubernetesDeploymentComponentFinalizer {
    fn new(finalize_key: si_cea::FinalizeKey) -> Self {
        Self { finalize_key }
    }
}

#[async_trait::async_trait]
impl si_cea::Finalize for KubernetesDeploymentComponentFinalizer {
    async fn finalize(
        &self,
        db: &si_data::Db,
        message: si_agent::WireMessage,
    ) -> si_cea::CeaResult<()> {
        let (_header, _qos, _response_header, object) = {
            let msg: si_agent::Message<crate::protobuf::KubernetesDeploymentComponent> =
                message.try_into()?;
            msg.into_parts()
        };

        object.finalize(db).await?;

        Ok(())
    }

    fn finalize_key(&self) -> si_cea::FinalizeKey {
        self.finalize_key.clone()
    }
}

impl si_cea::Finalizeable for KubernetesDeploymentComponentFinalizer {}

pub fn finalizer() -> si_cea::CeaResult<KubernetesDeploymentComponentFinalizer> {
    let mut builder = KubernetesDeploymentComponentFinalizerBuilder::new();
    builder.finalize_key(si_cea::FinalizeKey::new(builder.object_type()));

    builder.build()
}
