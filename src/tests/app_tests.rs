#[cfg(test)]
mod tests {
    use crate::docker::{Container, DockerClient};
    use crate::tui::App;

    fn create_test_container(name: &str, state: &str) -> Container {
        Container {
            id: "test_id".to_string(),
            image_id: "test_image_id".to_string(),
            names: vec![name.to_string()],
            image: "test_image".to_string(),
            command: "test_command".to_string(),
            created: 0,
            state: state.to_string(),
            status: "test_status".to_string(),
            ports: vec![],
            health: None,
        }
    }

    #[test]
    fn test_container_sorting() {
        let containers = vec![
            create_test_container("c", "stopped"),
            create_test_container("a", "running"),
            create_test_container("b", "running"),
            create_test_container("d", "stopped"),
        ];

        let client = DockerClient::new();
        let app = App::new(containers, client);

        // Verify containers are sorted: running first (alphabetically), then stopped (alphabetically)
        assert_eq!(app.containers[0].names[0], "a");
        assert_eq!(app.containers[1].names[0], "b");
        assert_eq!(app.containers[2].names[0], "c");
        assert_eq!(app.containers[3].names[0], "d");
    }

    #[test]
    fn test_container_health_status() {
        use crate::docker::models::Health;

        let mut container = create_test_container("test", "running");
        container.health = Some(Health {
            status: "healthy".to_string(),
            failing_streak: 0,
            log: vec![],
        });

        let client = DockerClient::new();
        let app = App::new(vec![container], client);

        assert_eq!(app.containers[0].health.as_ref().unwrap().status, "healthy");
    }
} 