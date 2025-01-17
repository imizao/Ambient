//! Used to implement all the server-specific host functions.
//!
//! If implementing a trait that is also available on the client, it should go in [super].

use std::str::FromStr;

use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    player::{is_player, user_id},
    runtime,
};
use ambient_ecs::{generated::messages::HttpResponse, query, EntityId, World};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::server::player_transport;

use super::super::Bindings;

use crate::shared::{
    self,
    conversion::FromBindgen,
    implementation::message,
    message::{MessageExt, Target},
};

mod physics;

#[async_trait::async_trait]
impl shared::wit::server_asset::Host for Bindings {}

#[async_trait::async_trait]
impl shared::wit::server_message::Host for Bindings {
    fn send(
        &mut self,
        target: shared::wit::server_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> wasm_bridge::Result<()> {
        use shared::wit::server_message::Target as WitTarget;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            WitTarget::ClientBroadcastUnreliable => {
                send_networked(world, None, module_id, name, data, false)
            }
            WitTarget::ClientBroadcastReliable => {
                send_networked(world, None, module_id, name, data, true)
            }
            WitTarget::ClientTargetedUnreliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, false)
            }
            WitTarget::ClientTargetedReliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, true)
            }
            WitTarget::LocalBroadcast(include_self) => {
                message::send_local(world, module_id, Target::All { include_self }, name, data)
            }
            WitTarget::Local(id) => message::send_local(
                world,
                module_id,
                Target::PackageOrModule(id.from_bindgen()),
                name,
                data,
            ),
        }
    }
}

fn send_networked(
    world: &World,
    target_user_id: Option<String>,
    module_id: EntityId,
    name: String,
    data: Vec<u8>,
    reliable: bool,
) -> wasm_bridge::Result<()> {
    let connections: Vec<_> = query((user_id(), player_transport()))
        .incl(is_player())
        .iter(world, None)
        .filter(|(_, (uid, _))| {
            target_user_id
                .as_ref()
                .map(|target_uid| target_uid == *uid)
                .unwrap_or(true)
        })
        .map(|(_, (_, connection))| connection.clone())
        .collect();

    for conn in connections {
        message::send_networked(world, conn, module_id, &name, &data, reliable)?;
    }

    Ok(())
}

impl shared::wit::server_http::Host for Bindings {
    fn get(&mut self, url: String) -> wasm_bridge::Result<()> {
        let id = self.id;
        let world = self.world_mut();
        let assets = world.resource(asset_cache());
        let runtime = world.resource(runtime());
        let async_run = world.resource(async_run()).clone();

        async fn make_request(url: String) -> wasm_bridge::Result<(u32, Vec<u8>)> {
            let response = reqwest::get(url).await?;
            Ok((
                response.status().as_u16() as u32,
                response.bytes().await?.to_vec(),
            ))
        }

        let resolved_url = AbsAssetUrl::from_str(&url)?
            .to_download_url(assets)?
            .to_string();

        runtime.spawn(async move {
            let result = make_request(resolved_url).await;
            let response = match result {
                Ok((status, body)) => HttpResponse {
                    url,
                    body,
                    status,
                    error: None,
                },
                Err(err) => HttpResponse {
                    url,
                    body: Vec::new(),
                    status: 0,
                    error: Some(err.to_string()),
                },
            };

            async_run.run(move |world| {
                response.send(world, Some(id)).unwrap();
            });
        });

        Ok(())
    }
}

impl shared::wit::server_ambient_package::Host for Bindings {
    fn load(&mut self, url: String) -> anyhow::Result<()> {
        ambient_package_semantic_native::add(self.world_mut(), url, false)?;
        Ok(())
    }
}
