//! AllStarLink Node Database (ASTDB) Downloader & Parser
//!
//! Parses the official AllStarLink `astdb.txt` / directory database format, which maps
//! 4-6 digit numeric node numbers to callsigns, locations, radio frequencies, and IP endpoints.
//!
//! Standard astdb record format:
//! `node|callsign|description|location|frequency|...`
//! Example:
//! `50000|W1AW|ARRL HQ Repeater|Newington, CT|146.520|`

use std::collections::BTreeMap;

/// Information record for an AllStarLink node in the ASTDB
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDbRecord {
    pub node: u32,
    pub callsign: String,
    pub description: String,
    pub location: String,
    pub frequency: Option<String>,
}

/// In-memory cache of the AllStarLink node directory
#[derive(Debug, Default, Clone)]
pub struct AstDb {
    records: BTreeMap<u32, AstDbRecord>,
}

impl AstDb {
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    /// Parses an `astdb.txt` formatted string into the database
    pub fn parse_text(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                if let Ok(node) = parts[0].parse::<u32>() {
                    let callsign = parts[1].trim().to_string();
                    let description = parts[2].trim().to_string();
                    let location = parts[3].trim().to_string();
                    let frequency = if parts.len() > 4 && !parts[4].trim().is_empty() {
                        Some(parts[4].trim().to_string())
                    } else {
                        None
                    };

                    self.records.insert(
                        node,
                        AstDbRecord {
                            node,
                            callsign,
                            description,
                            location,
                            frequency,
                        },
                    );
                }
            }
        }
    }

    pub fn lookup(&self, node: u32) -> Option<&AstDbRecord> {
        self.records.get(&node)
    }

    pub fn total_nodes(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astdb_parsing() {
        let sample = "\
; AllStarLink ASTDB Export
# Comment line
50000|W1AW|ARRL Headquarters Hub|Newington, CT|146.520|
2000|PARROT|AllStar Echo Server|Denver, CO||
1999|N0CALL|Local Desktop Testbed|Lab||
";

        let mut db = AstDb::new();
        db.parse_text(sample);

        assert_eq!(db.total_nodes(), 3);

        let n50000 = db.lookup(50000).expect("expected node 50000");
        assert_eq!(n50000.callsign, "W1AW");
        assert_eq!(n50000.description, "ARRL Headquarters Hub");
        assert_eq!(n50000.location, "Newington, CT");
        assert_eq!(n50000.frequency.as_deref(), Some("146.520"));

        let parrot = db.lookup(2000).expect("expected parrot node 2000");
        assert_eq!(parrot.callsign, "PARROT");
        assert_eq!(parrot.frequency, None);
    }
}
