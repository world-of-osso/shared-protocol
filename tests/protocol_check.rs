//! A server and a client with differing registries or type layouts connect over real
//! Netcode/UDP on localhost; both sides must reject the connection and drop the link, and
//! matching builds must verify.

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use std::net::UdpSocket;
use std::time::Instant;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use lightyear::prelude::client::{self as client_net, NetcodeClient};
use lightyear::prelude::server::{self as server_net, ClientOf, NetcodeServer, ServerUdpIo};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use shared::ProtocolPlugin;
use shared::protocol::{
    ProtocolCheckTimeout, ProtocolRegistrationExt, ProtocolRejected, ProtocolVerified,
    defer_lightyear_protocol_check,
};

const TICK: Duration = Duration::from_millis(10);

#[derive(Component, Serialize, Deserialize, PartialEq)]
struct AddedComponent(u32);

#[derive(Component, Serialize, Deserialize, PartialEq)]
struct OtherComponent(u32);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct AddedMessage(u32);

struct AddedChannel;

type Registrations = fn(&mut App);

fn no_registrations(_: &mut App) {}

fn with_added_component(app: &mut App) {
    app.protocol_component::<AddedComponent>().replicate();
}

fn with_both_components(app: &mut App) {
    app.protocol_component::<AddedComponent>().replicate();
    app.protocol_component::<OtherComponent>().replicate();
}

fn with_both_components_reversed(app: &mut App) {
    app.protocol_component::<OtherComponent>().replicate();
    app.protocol_component::<AddedComponent>().replicate();
}

fn with_added_message(app: &mut App) {
    app.register_protocol_message::<AddedMessage>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn with_v1_component(app: &mut App) {
    app.protocol_component::<layout_v1::LayoutComponent>()
        .replicate();
}

fn with_v2_component(app: &mut App) {
    app.protocol_component::<layout_v2::LayoutComponent>()
        .replicate();
}

fn with_v1_message(app: &mut App) {
    app.register_protocol_message::<layout_v1::LayoutMessage>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn with_v2_message(app: &mut App) {
    app.register_protocol_message::<layout_v2::LayoutMessage>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn with_added_channel(app: &mut App) {
    app.add_channel::<AddedChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
}

fn base_app(timeout: Duration) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.set_error_handler(defer_lightyear_protocol_check);
    app.insert_resource(ProtocolCheckTimeout(timeout));
    app
}

fn free_udp_port() -> u16 {
    UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
        .and_then(|socket| socket.local_addr())
        .expect("bind an ephemeral UDP port")
        .port()
}

fn server_app(port: u16, protocol: bool, extra: Registrations, timeout: Duration) -> App {
    let mut app = base_app(timeout);
    app.add_plugins(server_net::ServerPlugins {
        tick_duration: TICK,
    });
    if protocol {
        app.add_plugins(ProtocolPlugin);
    }
    extra(&mut app);
    record_server_verdicts(&mut app);
    app.finish();
    app.cleanup();
    let entity = app
        .world_mut()
        .spawn((
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)),
            ServerUdpIo::default(),
            NetcodeServer::new(server_net::NetcodeConfig::default()),
        ))
        .id();
    app.world_mut().trigger(server_net::Start { entity });
    app
}

fn client_app(port: u16, protocol: bool, extra: Registrations, timeout: Duration) -> (App, Entity) {
    let mut app = base_app(timeout);
    app.add_plugins(client_net::ClientPlugins {
        tick_duration: TICK,
    });
    if protocol {
        app.add_plugins(ProtocolPlugin);
    }
    extra(&mut app);
    app.finish();
    app.cleanup();
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let auth = Authentication::Manual {
        server_addr,
        client_id: 7,
        private_key: [0; 32],
        protocol_id: 0,
    };
    let netcode = NetcodeClient::new(auth, client_net::NetcodeConfig::default())
        .expect("construct Netcode client");
    let entity = app
        .world_mut()
        .spawn((
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)),
            PeerAddr(server_addr),
            UdpIo::default(),
            netcode,
        ))
        .id();
    app.world_mut().trigger(client_net::Connect { entity });
    (app, entity)
}

#[derive(Debug, Clone, PartialEq)]
enum Verdict {
    Verified,
    Rejected(String),
}

fn verdict(world: &mut World, entity: Entity) -> Option<Verdict> {
    let entity = world.entity(entity);
    if let Some(rejected) = entity.get::<ProtocolRejected>() {
        return Some(Verdict::Rejected(rejected.0.clone()));
    }
    entity
        .contains::<ProtocolVerified>()
        .then_some(Verdict::Verified)
}

/// Verdicts on the server's `ClientOf` entities, recorded as they land because a rejected
/// `ClientOf` is despawned.
#[derive(Resource, Default)]
struct ServerVerdicts(Vec<Verdict>);

fn record_server_verdicts(app: &mut App) {
    app.init_resource::<ServerVerdicts>();
    app.add_observer(
        |added: On<Add, ProtocolVerified>, mut verdicts: ResMut<ServerVerdicts>| {
            let _ = added;
            verdicts.0.push(Verdict::Verified);
        },
    );
    app.add_observer(
        |added: On<Add, ProtocolRejected>,
         rejected: Query<&ProtocolRejected>,
         mut verdicts: ResMut<ServerVerdicts>| {
            let reason = rejected.get(added.entity).expect("just added").0.clone();
            verdicts.0.push(Verdict::Rejected(reason));
        },
    );
}

fn server_connection_verdict(server: &App) -> Option<Verdict> {
    let verdicts = &server.world().resource::<ServerVerdicts>().0;
    assert!(
        verdicts.len() <= 1,
        "more than one server verdict: {verdicts:?}"
    );
    verdicts.first().cloned()
}

fn server_connections(server: &mut App) -> usize {
    server
        .world_mut()
        .query_filtered::<(), (With<ClientOf>, With<Connected>)>()
        .iter(server.world())
        .count()
}

struct Outcome {
    client: Option<Verdict>,
    server: Option<Verdict>,
    client_disconnected: bool,
    server_connections: usize,
    elapsed: Duration,
}

/// Steps both apps until each side reached a verdict and a rejected client finished
/// disconnecting, or `limit` passes.
fn run(
    server: (bool, Registrations),
    client: (bool, Registrations),
    timeout: Duration,
    limit: Duration,
) -> Outcome {
    let port = free_udp_port();
    let mut server_app = server_app(port, server.0, server.1, timeout);
    let (mut client_app, client_entity) = client_app(port, client.0, client.1, timeout);
    let started = Instant::now();
    loop {
        server_app.update();
        client_app.update();
        let outcome = Outcome {
            client: verdict(client_app.world_mut(), client_entity),
            server: server_connection_verdict(&server_app),
            server_connections: server_connections(&mut server_app),
            client_disconnected: client_app
                .world()
                .entity(client_entity)
                .contains::<Disconnected>(),
            elapsed: started.elapsed(),
        };
        let server_done = match outcome.server {
            Some(Verdict::Verified) => true,
            Some(Verdict::Rejected(_)) => {
                outcome.server_connections == 0 && outcome.client_disconnected
            }
            None => !server.0,
        };
        let client_done = match outcome.client {
            Some(Verdict::Verified) => true,
            Some(Verdict::Rejected(_)) => outcome.client_disconnected,
            None => !client.0,
        };
        if (server_done && client_done) || started.elapsed() > limit {
            return outcome;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn run_pair(server: Registrations, client: Registrations) -> Outcome {
    run(
        (true, server),
        (true, client),
        Duration::from_secs(10),
        Duration::from_secs(8),
    )
}

fn assert_rejected(outcome: Outcome, registry: &str) {
    let expected = format!(
        "Client and server protocols differ ({registry} registry). Rebuild both from the same shared-protocol revision."
    );
    assert_eq!(outcome.client, Some(Verdict::Rejected(expected.clone())));
    assert_eq!(outcome.server, Some(Verdict::Rejected(expected)));
    assert!(
        outcome.client_disconnected,
        "rejected client stayed connected"
    );
    assert_eq!(
        outcome.server_connections, 0,
        "server kept serving the client"
    );
}

#[test]
fn matching_builds_verify_on_both_sides() {
    let outcome = run_pair(with_added_component, with_added_component);
    assert_eq!(outcome.client, Some(Verdict::Verified));
    assert_eq!(outcome.server, Some(Verdict::Verified));
    assert!(!outcome.client_disconnected);
    assert_eq!(outcome.server_connections, 1);
}

#[test]
fn component_registered_only_on_client_is_rejected_by_both_sides() {
    assert_rejected(
        run_pair(no_registrations, with_added_component),
        "component",
    );
}

#[test]
fn component_registration_order_difference_is_rejected_by_both_sides() {
    assert_rejected(
        run_pair(with_both_components, with_both_components_reversed),
        "component",
    );
}

#[test]
fn message_registered_only_on_server_is_rejected_by_both_sides() {
    assert_rejected(run_pair(with_added_message, no_registrations), "message");
}

#[test]
fn component_with_an_added_field_is_rejected_by_both_sides() {
    let outcome = run_pair(with_v1_component, with_v2_component);
    assert_rejected(outcome, "component layout");
}

#[test]
fn message_with_an_added_field_is_rejected_by_both_sides() {
    assert_rejected(run_pair(with_v2_message, with_v1_message), "message layout");
}

#[test]
fn builds_with_the_same_layouts_verify() {
    let outcome = run_pair(with_v1_component, with_v1_component);
    assert_eq!(outcome.client, Some(Verdict::Verified));
    assert_eq!(outcome.server, Some(Verdict::Verified));
}

#[test]
fn channel_registered_only_on_client_is_rejected_by_both_sides() {
    assert_rejected(run_pair(no_registrations, with_added_channel), "channel");
}

#[test]
fn client_rejects_a_server_that_never_sends_a_fingerprint() {
    let outcome = run(
        (false, no_registrations),
        (true, no_registrations),
        Duration::from_millis(800),
        Duration::from_secs(8),
    );
    assert_eq!(
        outcome.client,
        Some(Verdict::Rejected(
            "No protocol fingerprint from the peer within 0.8 s: client and server builds are incompatible."
                .into()
        ))
    );
    assert!(outcome.client_disconnected);
}

#[test]
fn server_rejects_a_client_that_never_sends_a_fingerprint() {
    let outcome = run(
        (true, no_registrations),
        (false, no_registrations),
        Duration::from_millis(800),
        Duration::from_secs(8),
    );
    assert_eq!(
        outcome.server,
        Some(Verdict::Rejected(
            "No protocol fingerprint from the peer within 0.8 s: client and server builds are incompatible."
                .into()
        ))
    );
    assert_eq!(
        outcome.server_connections, 0,
        "server kept serving the client"
    );
    // The client runs no check, so only the server's disconnect ends its link: 0.8 s check
    // timeout + 1 s grace, where the 3 s netcode timeout would land past 4.8 s.
    assert!(
        outcome.client_disconnected,
        "server never disconnected the client"
    );
    assert!(
        outcome.elapsed < Duration::from_secs(3),
        "client disconnected only after {:?}, at the netcode timeout",
        outcome.elapsed
    );
}
