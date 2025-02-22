#[cfg(test)]
mod tests {
    use crate::docker::DockerClient;

    #[tokio::test]
    async fn test_list_containers() {
        let client = DockerClient::new();
        let result = client.list_containers().await;
        
        // We can't guarantee containers will be present, but we can verify the call works
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_containers_blocking() {
        let client = DockerClient::new();
        let result = client.list_containers_blocking();
        
        // We can't guarantee containers will be present, but we can verify the call works
        assert!(result.is_ok());
    }
} 