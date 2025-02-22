#[cfg(test)]
mod tests {
    use crate::docker::Port;
    use crate::tui::format_ports;

    #[test]
    fn test_format_ports() {
        // Test empty ports
        let empty_ports: Vec<Port> = vec![];
        assert_eq!(format_ports(&empty_ports), "None");

        // Test single port without external mapping
        let internal_only = vec![Port {
            ip: None,
            internal: 80,
            external: None,
            protocol: "tcp".to_string(),
        }];
        assert_eq!(format_ports(&internal_only), "80/tcp");

        // Test port with external mapping
        let mapped_port = vec![Port {
            ip: Some("0.0.0.0".to_string()),
            internal: 80,
            external: Some(8080),
            protocol: "tcp".to_string(),
        }];
        assert_eq!(format_ports(&mapped_port), "0.0.0.0:8080:80/tcp");

        // Test multiple ports
        let multiple_ports = vec![
            Port {
                ip: None,
                internal: 80,
                external: Some(8080),
                protocol: "tcp".to_string(),
            },
            Port {
                ip: None,
                internal: 443,
                external: Some(8443),
                protocol: "tcp".to_string(),
            },
        ];
        assert_eq!(format_ports(&multiple_ports), "8080:80/tcp, 8443:443/tcp");
    }
} 