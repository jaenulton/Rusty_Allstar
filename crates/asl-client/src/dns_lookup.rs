//! AllStarLink Node DNS / IP Resolution (nodes.allstarlink.org)
//!
//! Resolves numeric AllStarLink node numbers (e.g. 50000) to host IP and IAX2 port
//! via AllStarLink DNS SRV / TXT records or directory queries.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Node lookup result containing target IP and port
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLocation {
    pub node: u32,
    pub socket_addr: SocketAddr,
}

/// Resolves standard ASL node hostname convention (e.g., node 50000 -> 50000.nodes.allstarlink.org)
pub fn format_asl_node_hostname(node: u32) -> String {
    format!("{}.nodes.allstarlink.org", node)
}

/// Fallback / simulated node resolver for offline development & known public nodes
pub fn resolve_node_stub(node: u32) -> Option<NodeLocation> {
    match node {
        // Known test nodes / parrots / hubs
        1999 => Some(NodeLocation {
            node,
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4569),
        }),
        // Standard WAN AllStarLink Echo/Parrot
        2000 => Some(NodeLocation {
            node,
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)), 4569),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asl_node_hostname_formatting() {
        assert_eq!(format_asl_node_hostname(50000), "50000.nodes.allstarlink.org");
        assert_eq!(format_asl_node_hostname(2521), "2521.nodes.allstarlink.org");
    }

    #[test]
    fn test_node_lookup_stub() {
        let loc = resolve_node_stub(1999).expect("expected node 1999");
        assert_eq!(loc.socket_addr.port(), 4569);
    }
}
