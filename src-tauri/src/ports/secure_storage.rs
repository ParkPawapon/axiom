use crate::shared::result::app_result::AppResult;

pub trait SecureStorage: Send + Sync {
    fn store_secret(&self, namespace: &str, key: &str, secret: &str) -> AppResult<()>;

    fn get_secret(&self, namespace: &str, key: &str) -> AppResult<Option<String>>;

    fn delete_secret(&self, namespace: &str, key: &str) -> AppResult<()>;
}
