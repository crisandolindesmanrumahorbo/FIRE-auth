use super::service::AuthService;

pub struct AuthController {
    service: AuthService,
}

impl AuthController {
    pub fn new(service: AuthService) -> Self {
        AuthController { service }
    }

    pub async fn login(&self, request: &str) -> (String, String) {
        self.service.login(request).await
    }

    pub async fn register(&self, request: &str) -> (String, String) {
        self.service.register(request).await
    }
}
