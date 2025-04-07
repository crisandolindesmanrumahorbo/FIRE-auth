use super::service::AuthService;

pub struct AuthController {
    service: AuthService,
}

impl AuthController {
    pub fn new(pool: sqlx::AnyPool) -> Self {
        AuthController {
            service: AuthService::new(super::repository::AuthRepository::new(pool)),
        }
    }

    pub async fn login(&self, request: &str) -> (String, String) {
        self.service.login(request).await
    }

    pub async fn register(&self, request: &str) -> (String, String) {
        self.service.register(request).await
    }

    pub fn validate(&self, request: &str) -> (String, String) {
        self.service.validate(request)
    }
}
