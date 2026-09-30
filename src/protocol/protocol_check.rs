//! Connect-time protocol check. Each peer sends its registry hashes when the connection
//! comes up and drops a peer whose hashes differ or never arrive.
//!
//! Lightyear 0.28's own `ProtocolCheck` misses component differences: its
//! `ComponentRegistry` hasher is never fed since replication moved to replicon, and
//! lightyear runs replicon with `AuthMethod::None`, so replicon's `ProtocolHash` is never
//! compared either. It also only checks on the client.

use core::time::Duration;

use bevy::ecs::error::{ErrorContext, match_severity};
use bevy::prelude::*;
use bevy_replicon::shared::protocol::ProtocolHash;
use lightyear::connection::client::Disconnecting;
use lightyear::prelude::server::ClientOf;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

/// Registry hashes of one build, each covering type names in registration order.
/// `components` is replicon's hash of the replication rules and replicon events.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolFingerprint {
    pub messages: u64,
    pub channels: u64,
    pub components: ProtocolHash,
}

/// Reliable ordered channel for `ProtocolFingerprint`, bidirectional. Registered before
/// every other shared channel so its id survives registry differences.
pub struct ProtocolCheckChannel;

/// The peer on this connection sent a matching `ProtocolFingerprint`.
#[derive(Component, Debug)]
pub struct ProtocolVerified;

/// The peer on this connection runs a different protocol; the link is dropped shortly after.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRejected(pub String);

/// How long a connection may wait for the peer's `ProtocolFingerprint`. A peer built
/// before this check never sends one.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ProtocolCheckTimeout(pub Duration);

impl Default for ProtocolCheckTimeout {
    fn default() -> Self {
        Self(Duration::from_secs(10))
    }
}

/// `Time<Real>` elapsed when the connection came up.
#[derive(Component)]
struct ProtocolCheckStarted(Duration);

/// How long a rejected link stays up so this side's fingerprint, which may still be in
/// reliable resend, reaches the peer and both sides report the mismatch.
const REJECT_GRACE: Duration = Duration::from_secs(1);

/// `Time<Real>` elapsed at which a rejected link is dropped.
#[derive(Component)]
struct ProtocolCutoff(Duration);

pub(super) fn register_protocol_check(app: &mut App) {
    app.add_channel::<ProtocolCheckChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<ProtocolFingerprint>()
        .add_direction(NetworkDirection::Bidirectional);
    app.init_resource::<ProtocolCheckTimeout>();
    app.add_observer(reset_check);
    app.add_systems(PostUpdate, send_fingerprints.before(MessageSystems::Send));
    app.add_systems(
        PreUpdate,
        (verify_fingerprints, expire_unverified, drop_rejected_links)
            .chain()
            .after(MessageSystems::Receive),
    );
}

fn local_fingerprint(
    messages: &mut MessageRegistry,
    channels: &mut ChannelRegistry,
    components: ProtocolHash,
) -> ProtocolFingerprint {
    ProtocolFingerprint {
        messages: messages.finish(),
        channels: channels.finish(),
        components,
    }
}

/// A reconnecting client starts a new check; the verdict of its previous connection stays
/// readable until then.
fn reset_check(connecting: On<Add, Connecting>, mut commands: Commands) {
    commands.entity(connecting.entity).remove::<(
        ProtocolVerified,
        ProtocolRejected,
        ProtocolCutoff,
        ProtocolCheckStarted,
    )>();
}

/// A system rather than an `Add<Connected>` observer: a link can connect before
/// `App::finish` inserts `ProtocolHash`, and systems only run after it.
fn send_fingerprints(
    mut connected: Query<
        (Entity, Option<&mut MessageSender<ProtocolFingerprint>>),
        Added<Connected>,
    >,
    mut messages: ResMut<MessageRegistry>,
    mut channels: ResMut<ChannelRegistry>,
    components: Res<ProtocolHash>,
    time: Res<Time<Real>>,
    mut commands: Commands,
) {
    for (entity, sender) in &mut connected {
        commands
            .entity(entity)
            .insert(ProtocolCheckStarted(time.elapsed()));
        let Some(mut sender) = sender else {
            warn!("connection {entity} has no ProtocolFingerprint sender; the peer will time out");
            continue;
        };
        sender.send::<ProtocolCheckChannel>(local_fingerprint(
            &mut messages,
            &mut channels,
            *components,
        ));
    }
}

fn verify_fingerprints(
    mut receivers: Query<(Entity, &mut MessageReceiver<ProtocolFingerprint>)>,
    mut messages: ResMut<MessageRegistry>,
    mut channels: ResMut<ChannelRegistry>,
    components: Res<ProtocolHash>,
    time: Res<Time<Real>>,
    mut commands: Commands,
) {
    for (entity, mut receiver) in &mut receivers {
        for remote in receiver.receive() {
            let local = local_fingerprint(&mut messages, &mut channels, *components);
            match describe_mismatch(&local, &remote) {
                None => {
                    commands.entity(entity).insert(ProtocolVerified);
                }
                Some(reason) => {
                    error!(
                        "rejecting connection {entity}: {reason} (local {local:?}, peer {remote:?})"
                    );
                    reject(&mut commands, entity, reason, time.elapsed());
                }
            }
        }
    }
}

type AwaitingFingerprint = (
    With<Connected>,
    Without<ProtocolVerified>,
    Without<ProtocolRejected>,
);

fn expire_unverified(
    pending: Query<(Entity, &ProtocolCheckStarted), AwaitingFingerprint>,
    time: Res<Time<Real>>,
    timeout: Res<ProtocolCheckTimeout>,
    mut commands: Commands,
) {
    for (entity, started) in &pending {
        if time.elapsed().saturating_sub(started.0) < timeout.0 {
            continue;
        }
        let reason = format!(
            "No protocol fingerprint from the peer within {} s: client and server builds are incompatible.",
            timeout.0.as_secs_f32()
        );
        error!("rejecting connection {entity}: {reason}");
        reject(&mut commands, entity, reason, time.elapsed());
    }
}

fn reject(commands: &mut Commands, entity: Entity, reason: String, now: Duration) {
    commands
        .entity(entity)
        .insert((ProtocolRejected(reason), ProtocolCutoff(now + REJECT_GRACE)));
}

/// A client disconnects through its transport. Lightyear 0.28 has no per-client disconnect
/// on the server (`Disconnect` on a netcode `ClientOf` is not observed), so the server stops
/// serving the link: `Disconnecting` drops `Connected`, which ends payloads and keepalives,
/// and lightyear despawns the `ClientOf` in `Last`.
fn drop_rejected_links(
    rejected: Query<(Entity, &ProtocolCutoff, Has<ClientOf>), With<Connected>>,
    time: Res<Time<Real>>,
    mut commands: Commands,
) {
    for (entity, cutoff, server_side) in &rejected {
        if time.elapsed() < cutoff.0 {
            continue;
        }
        commands.entity(entity).remove::<ProtocolCutoff>();
        if server_side {
            commands.entity(entity).insert(Disconnecting);
        } else {
            commands.trigger(Disconnect { entity });
        }
    }
}

const LIGHTYEAR_PROTOCOL_CHECK: &str =
    "lightyear::protocol::ProtocolCheckPlugin::receive_verify_protocol";

/// App error handler that leaves protocol differences to this check. Lightyear's own
/// client-side check fails through the error handler, which by default panics the app on
/// a message or channel difference before this check can report it.
pub fn defer_lightyear_protocol_check(error: BevyError, ctx: ErrorContext) {
    if ctx.name().to_string() == LIGHTYEAR_PROTOCOL_CHECK {
        warn!("lightyear protocol check failed ({error}); the shared protocol check reports it");
        return;
    }
    match_severity(error, ctx);
}

/// User-facing reason naming every registry that differs, or `None` when all match.
fn describe_mismatch(local: &ProtocolFingerprint, remote: &ProtocolFingerprint) -> Option<String> {
    let differing: Vec<&str> = [
        ("message", local.messages != remote.messages),
        ("component", local.components != remote.components),
        ("channel", local.channels != remote.channels),
    ]
    .into_iter()
    .filter_map(|(registry, differs)| differs.then_some(registry))
    .collect();
    if differing.is_empty() {
        return None;
    }
    Some(format!(
        "Client and server protocols differ ({} registry). Rebuild both from the same shared-protocol revision.",
        differing.join(", ")
    ))
}
