//! Wire layout of every registered message and replicated component, for the protocol
//! check. The registries it runs next to hash type names in registration order, so a field
//! added to a type keeps them equal; this hashes the serde shape (fields, their types and
//! order, enum variants) that the payload encoding follows.

use bevy::prelude::*;
use lightyear::prelude::{
    AppComponentExt, AppMessageExt, ComponentRegistry, Message, MessageRegistry,
};
use lightyear::utils::registry::{TypeKind, TypeMapper};
use lightyear_messages::registry::{MessageKind, MessageRegistration};
use lightyear_replication::registry::ComponentKind;
use lightyear_replication::registry::replication::ComponentRegistration;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

use super::layout_tracer::trace_layout;

/// Layout hash of each type registered through [`ProtocolRegistrationExt`].
#[derive(Resource, Default)]
struct ProtocolLayouts {
    messages: HashMap<MessageKind, u64>,
    components: HashMap<ComponentKind, u64>,
}

/// Registration that also records the type's wire layout. Every shared message and
/// replicated component registers through this; the protocol check refuses to start
/// with a type registered around it.
pub trait ProtocolRegistrationExt {
    fn register_protocol_message<M: Message + Serialize + DeserializeOwned>(
        &mut self,
    ) -> MessageRegistration<'_, M>;

    fn protocol_component<C: Component + Serialize + DeserializeOwned>(
        &mut self,
    ) -> ComponentRegistration<'_, C>;
}

impl ProtocolRegistrationExt for App {
    fn register_protocol_message<M: Message + Serialize + DeserializeOwned>(
        &mut self,
    ) -> MessageRegistration<'_, M> {
        let layout = layout_hash::<M>();
        self.world_mut()
            .get_resource_or_init::<ProtocolLayouts>()
            .messages
            .insert(MessageKind::of::<M>(), layout);
        self.register_message::<M>()
    }

    fn protocol_component<C: Component + Serialize + DeserializeOwned>(
        &mut self,
    ) -> ComponentRegistration<'_, C> {
        let layout = layout_hash::<C>();
        self.world_mut()
            .get_resource_or_init::<ProtocolLayouts>()
            .components
            .insert(ComponentKind::of::<C>(), layout);
        self.component::<C>()
    }
}

/// Message and component layout hashes, each folded in registration order.
#[derive(Resource, Clone, Copy, Debug)]
pub(super) struct LocalLayouts {
    pub messages: u64,
    pub components: u64,
}

/// Panics naming a message or component that was registered without its layout: its
/// fields would escape the check.
pub(super) fn finish_layouts(world: &mut World) -> LocalLayouts {
    world.init_resource::<ProtocolLayouts>();
    let layouts = world.resource::<ProtocolLayouts>();
    let messages = &world.resource::<MessageRegistry>().kind_map;
    let components = world
        .get_resource::<ComponentRegistry>()
        .map_or(0, |registry| {
            fold_registered(
                &registry.kind_map,
                &layouts.components,
                "protocol_component",
            )
        });
    LocalLayouts {
        messages: fold_registered(messages, &layouts.messages, "register_protocol_message"),
        components,
    }
}

/// Folds the layouts in registration order. Lightyear's own types are skipped: their
/// names are in the registry hashes and the lightyear version fixes their layout.
fn fold_registered<K: TypeKind>(
    mapper: &TypeMapper<K>,
    layouts: &HashMap<K, u64>,
    register_with: &str,
) -> u64 {
    let registered = (0..).map_while(|net_id| mapper.kind(net_id));
    fold(registered.filter_map(|kind| {
        let name = mapper.name(kind).unwrap_or("?");
        match layouts.get(kind) {
            Some(layout) => Some(*layout),
            None if name.starts_with("lightyear") => None,
            None => panic!("{name} is registered without its layout; use {register_with}"),
        }
    }))
}

fn fold(hashes: impl IntoIterator<Item = u64>) -> u64 {
    hashes.into_iter().fold(FNV_OFFSET, |hash, layout| {
        fnv1a(hash, &layout.to_le_bytes())
    })
}

/// Hash of `T`'s serde layout, with the shapes of every type it contains.
fn layout_hash<T: DeserializeOwned>() -> u64 {
    let layout = trace_layout::<T>().unwrap_or_else(|error| {
        panic!(
            "cannot trace the wire layout of {}: {error}",
            core::any::type_name::<T>()
        )
    });
    fnv1a(FNV_OFFSET, layout.as_bytes())
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a: fixed across Rust releases, unlike `DefaultHasher`.
fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
#[path = "protocol_layout_tests.rs"]
mod tests;
