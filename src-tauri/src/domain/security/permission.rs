#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Permission {
    ReadConfig,
    WriteConfig,
    ManageServices,
    ManageCertificates,
}
