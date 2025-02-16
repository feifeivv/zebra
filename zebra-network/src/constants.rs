//! Definitions of constants.

use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;

// XXX should these constants be split into protocol also?
use crate::protocol::external::types::*;

use zebra_chain::{parameters::NetworkUpgrade, serialization::Duration32};

/// The buffer size for the peer set.
///
/// This should be greater than 1 to avoid sender contention, but also reasonably
/// small, to avoid queueing too many in-flight block downloads. (A large queue
/// of in-flight block downloads can choke a constrained local network
/// connection, or a small peer set on testnet.)
///
/// We assume that Zebra nodes have at least 10 Mbps bandwidth. Therefore, a
/// maximum-sized block can take up to 2 seconds to download. So the peer set
/// buffer adds up to 6 seconds worth of blocks to the queue.
pub const PEERSET_BUFFER_SIZE: usize = 3;

/// The timeout for requests made to a remote peer.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// The timeout for handshakes when connecting to new peers.
///
/// This timeout should remain small, because it helps stop slow peers getting
/// into the peer set. This is particularly important for network-constrained
/// nodes, and on testnet.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(4);

/// We expect to receive a message from a live peer at least once in this time duration.
///
/// This is the sum of:
/// - the interval between connection heartbeats
/// - the timeout of a possible pending (already-sent) request
/// - the timeout for a possible queued request
/// - the timeout for the heartbeat request itself
///
/// This avoids explicit synchronization, but relies on the peer
/// connector actually setting up channels and these heartbeats in a
/// specific manner that matches up with this math.
pub const MIN_PEER_RECONNECTION_DELAY: Duration = Duration::from_secs(60 + 20 + 20 + 20);

/// The maximum duration since a peer was last seen to consider it reachable.
///
/// This is used to prevent Zebra from gossiping addresses that are likely unreachable. Peers that
/// have last been seen more than this duration ago will not be gossiped.
///
/// This is determined as a tradeoff between network health and network view leakage. From the
/// [Bitcoin protocol documentation](https://en.bitcoin.it/wiki/Protocol_documentation#getaddr):
///
/// "The typical presumption is that a node is likely to be active if it has been sending a message
/// within the last three hours."
pub const MAX_PEER_ACTIVE_FOR_GOSSIP: Duration32 = Duration32::from_hours(3);

/// Regular interval for sending keepalive `Ping` messages to each
/// connected peer.
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

/// The minimum time between successive calls to [`CandidateSet::next()`][Self::next].
///
/// ## Security
///
/// Zebra resists distributed denial of service attacks by making sure that new peer connections
/// are initiated at least `MIN_PEER_CONNECTION_INTERVAL` apart.
pub const MIN_PEER_CONNECTION_INTERVAL: Duration = Duration::from_millis(100);

/// The minimum time between successive calls to [`CandidateSet::update()`][Self::update].
///
/// ## Security
///
/// Zebra resists distributed denial of service attacks by making sure that requests for more
/// peer addresses are sent at least `MIN_PEER_GET_ADDR_INTERVAL` apart.
pub const MIN_PEER_GET_ADDR_INTERVAL: Duration = Duration::from_secs(10);

/// The number of GetAddr requests sent when crawling for new peers.
///
/// ## SECURITY
///
/// The fanout should be greater than 2, so that Zebra avoids getting a majority
/// of its initial address book entries from a single peer.
///
/// Zebra regularly crawls for new peers, initiating a new crawl every
/// [`crawl_new_peer_interval`](crate::config::Config.crawl_new_peer_interval).
///
/// TODO: limit the number of addresses that Zebra uses from a single peer
///       response (#1869)
pub const GET_ADDR_FANOUT: usize = 3;

/// Truncate timestamps in outbound address messages to this time interval.
///
/// ## SECURITY
///
/// Timestamp truncation prevents a peer from learning exactly when we received
/// messages from each of our peers.
pub const TIMESTAMP_TRUNCATION_SECONDS: u32 = 30 * 60;

/// The User-Agent string provided by the node.
///
/// This must be a valid [BIP 14] user agent.
///
/// [BIP 14]: https://github.com/bitcoin/bips/blob/master/bip-0014.mediawiki
//
// TODO: generate this from crate metadata (#2375)
pub const USER_AGENT: &str = "/Zebra:1.0.0-alpha.15/";

/// The Zcash network protocol version implemented by this crate, and advertised
/// during connection setup.
///
/// The current protocol version is checked by our peers. If it is too old,
/// newer peers will disconnect from us.
///
/// The current protocol version typically changes before Mainnet and Testnet
/// network upgrades.
pub const CURRENT_NETWORK_PROTOCOL_VERSION: Version = Version(170_013);

/// The minimum network protocol version accepted by this crate for each network,
/// represented as a network upgrade.
///
/// The minimum protocol version is used to check the protocol versions of our
/// peers during the initial block download. After the intial block download,
/// we use the current block height to select the minimum network protocol
/// version.
///
/// If peer versions are too old, we will disconnect from them.
///
/// The minimum network protocol version typically changes after Mainnet network
/// upgrades.
pub const INITIAL_MIN_NETWORK_PROTOCOL_VERSION: NetworkUpgrade = NetworkUpgrade::Canopy;

/// The default RTT estimate for peer responses.
///
/// We choose a high value for the default RTT, so that new peers must prove they
/// are fast, before we prefer them to other peers. This is particularly
/// important on testnet, which has a small number of peers, which are often
/// slow.
///
/// Make the default RTT slightly higher than the request timeout.
pub const EWMA_DEFAULT_RTT: Duration = Duration::from_secs(REQUEST_TIMEOUT.as_secs() + 1);

/// The decay time for the EWMA response time metric used for load balancing.
///
/// This should be much larger than the `SYNC_RESTART_TIMEOUT`, so we choose
/// better peers when we restart the sync.
pub const EWMA_DECAY_TIME: Duration = Duration::from_secs(200);

lazy_static! {
    /// OS-specific error when the port attempting to be opened is already in use.
    pub static ref PORT_IN_USE_ERROR: Regex = if cfg!(unix) {
        #[allow(clippy::trivial_regex)]
        Regex::new(&regex::escape("already in use"))
    } else {
        Regex::new("(access a socket in a way forbidden by its access permissions)|(Only one usage of each socket address)")
    }.expect("regex is valid");
}

/// The timeout for DNS lookups.
///
/// [6.1.3.3 Efficient Resource Usage] from [RFC 1123: Requirements for Internet Hosts]
/// suggest no less than 5 seconds for resolving timeout.
///
/// [RFC 1123: Requirements for Internet Hosts] https://tools.ietf.org/rfcmarkup?doc=1123
/// [6.1.3.3  Efficient Resource Usage] https://tools.ietf.org/rfcmarkup?doc=1123#page-77
pub const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Magic numbers used to identify different Zcash networks.
pub mod magics {
    use super::*;
    /// The production mainnet.
    pub const MAINNET: Magic = Magic([0x24, 0xe9, 0x27, 0x64]);
    /// The testnet.
    pub const TESTNET: Magic = Magic([0xfa, 0x1a, 0xf9, 0xbf]);
}

#[cfg(test)]
mod tests {

    use super::*;

    /// This assures that the `Duration` value we are computing for
    /// MIN_PEER_RECONNECTION_DELAY actually matches the other const values it
    /// relies on.
    #[test]
    fn ensure_live_peer_duration_value_matches_others() {
        zebra_test::init();

        let constructed_live_peer_duration =
            HEARTBEAT_INTERVAL + REQUEST_TIMEOUT + REQUEST_TIMEOUT + REQUEST_TIMEOUT;

        assert_eq!(MIN_PEER_RECONNECTION_DELAY, constructed_live_peer_duration);
    }

    /// Make sure that the timeout values are consistent with each other.
    #[test]
    fn ensure_timeouts_consistent() {
        zebra_test::init();

        assert!(HANDSHAKE_TIMEOUT <= REQUEST_TIMEOUT,
                "Handshakes are requests, so the handshake timeout can't be longer than the timeout for all requests.");
        // This check is particularly important on testnet, which has a small
        // number of peers, which are often slow.
        assert!(EWMA_DEFAULT_RTT > REQUEST_TIMEOUT,
                "The default EWMA RTT should be higher than the request timeout, so new peers are required to prove they are fast, before we prefer them to other peers.");

        assert!(EWMA_DECAY_TIME > REQUEST_TIMEOUT,
                "The EWMA decay time should be higher than the request timeout, so timed out peers are penalised by the EWMA.");
    }
}
