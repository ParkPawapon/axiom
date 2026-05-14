use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use directories::{BaseDirs, ProjectDirs};

use crate::domain::networking::ssl_certificate::{
    CertificateTrustResult, CertificateTrustStatus, LocalCertificate,
};
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::domain::security::elevation::{PermissionElevationKind, PermissionElevationRequest};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::certificate_manager::CertificateManager;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_local_domain::validate_local_domain;

const CERTIFICATE_TIMEOUT: Duration = Duration::from_secs(60);
const OUTPUT_LIMIT_BYTES: usize = 64 * 1024;
const CA_COMMON_NAME: &str = "AxiomPHP Local Development Root CA";

#[derive(Debug, Clone)]
pub struct LocalCertificateManager {
    certificate_root: PathBuf,
}

impl LocalCertificateManager {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self {
            certificate_root: project_dirs
                .data_local_dir()
                .join("security")
                .join("certificates"),
        })
    }

    pub fn with_certificate_root(certificate_root: PathBuf) -> Self {
        Self { certificate_root }
    }

    pub fn certificate_authority_path(&self) -> PathBuf {
        self.certificate_root.join("axiomphp-local-ca.crt")
    }

    fn certificate_authority_key_path(&self) -> PathBuf {
        self.certificate_root.join("axiomphp-local-ca.key")
    }

    fn domain_dir(&self, domain: &str) -> PathBuf {
        self.certificate_root
            .join("domains")
            .join(domain.replace('.', "_"))
    }

    fn openssl_runner() -> AppResult<(CommandRunner, PathBuf)> {
        let Some(openssl_path) = ExecutableResolver::from_env().resolve("openssl") else {
            return Err(AppError::Configuration(
                "OpenSSL executable was not found on PATH".to_string(),
            ));
        };
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([openssl_path.clone()])
                .with_default_timeout(CERTIFICATE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        Ok((runner, openssl_path))
    }

    fn ensure_certificate_root(&self) -> AppResult<()> {
        fs::create_dir_all(&self.certificate_root).map_err(|error| {
            AppError::Infrastructure(format!("failed to create certificate directory: {error}"))
        })?;
        lock_private_directory(&self.certificate_root)
    }

    fn ensure_certificate_authority(
        &self,
        runner: &CommandRunner,
        openssl_path: &Path,
    ) -> AppResult<()> {
        let ca_key = self.certificate_authority_key_path();
        let ca_cert = self.certificate_authority_path();

        if ca_key.exists() && ca_cert.exists() {
            return Ok(());
        }

        run_openssl(
            runner,
            openssl_path,
            ["genrsa", "-out", &ca_key.to_string_lossy(), "4096"],
        )?;
        run_openssl(
            runner,
            openssl_path,
            [
                "req",
                "-x509",
                "-new",
                "-nodes",
                "-key",
                &ca_key.to_string_lossy(),
                "-sha256",
                "-days",
                "3650",
                "-subj",
                &format!("/CN={CA_COMMON_NAME}"),
                "-out",
                &ca_cert.to_string_lossy(),
            ],
        )?;
        lock_private_file(&ca_key)?;

        Ok(())
    }
}

impl CertificateManager for LocalCertificateManager {
    fn generate_local_certificate(&self, domain: &str) -> AppResult<LocalCertificate> {
        let domain = validate_local_domain(domain)?;
        self.ensure_certificate_root()?;
        let (runner, openssl_path) = Self::openssl_runner()?;
        self.ensure_certificate_authority(&runner, &openssl_path)?;

        let domain_dir = self.domain_dir(&domain);
        fs::create_dir_all(&domain_dir).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to create domain certificate directory: {error}"
            ))
        })?;
        lock_private_directory(&domain_dir)?;

        let key_path = domain_dir.join(format!("{domain}.key"));
        let csr_path = domain_dir.join(format!("{domain}.csr"));
        let certificate_path = domain_dir.join(format!("{domain}.crt"));
        let config_path = domain_dir.join("openssl.cnf");

        fs::write(&config_path, openssl_domain_config(&domain)).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write OpenSSL certificate config: {error}"
            ))
        })?;

        if !key_path.exists() {
            run_openssl(
                &runner,
                &openssl_path,
                ["genrsa", "-out", &key_path.to_string_lossy(), "2048"],
            )?;
            lock_private_file(&key_path)?;
        }

        run_openssl(
            &runner,
            &openssl_path,
            [
                "req",
                "-new",
                "-key",
                &key_path.to_string_lossy(),
                "-subj",
                &format!("/CN={domain}"),
                "-out",
                &csr_path.to_string_lossy(),
                "-config",
                &config_path.to_string_lossy(),
            ],
        )?;
        run_openssl(
            &runner,
            &openssl_path,
            [
                "x509",
                "-req",
                "-in",
                &csr_path.to_string_lossy(),
                "-CA",
                &self.certificate_authority_path().to_string_lossy(),
                "-CAkey",
                &self.certificate_authority_key_path().to_string_lossy(),
                "-CAcreateserial",
                "-out",
                &certificate_path.to_string_lossy(),
                "-days",
                "825",
                "-sha256",
                "-extensions",
                "req_ext",
                "-extfile",
                &config_path.to_string_lossy(),
            ],
        )?;

        Ok(LocalCertificate {
            domain,
            certificate_path: certificate_path.to_string_lossy().into_owned(),
            private_key_path: key_path.to_string_lossy().into_owned(),
            certificate_authority_path: self
                .certificate_authority_path()
                .to_string_lossy()
                .into_owned(),
            openssl_config_path: config_path.to_string_lossy().into_owned(),
            issued_at: Utc::now(),
            status_message: "Local certificate and private key were generated with an app-owned certificate authority.".to_string(),
        })
    }

    fn inspect_trust_status(&self) -> AppResult<CertificateTrustResult> {
        let ca_path = self.certificate_authority_path();

        if !ca_path.exists() {
            return Ok(CertificateTrustResult {
                certificate_authority_path: ca_path.to_string_lossy().into_owned(),
                status: CertificateTrustStatus::Missing,
                requires_elevation: false,
                elevation: None,
                status_message: "Local certificate authority has not been generated yet."
                    .to_string(),
            });
        }

        let trusted = certificate_authority_is_trusted(&ca_path)?;

        Ok(CertificateTrustResult {
            certificate_authority_path: ca_path.to_string_lossy().into_owned(),
            status: if trusted {
                CertificateTrustStatus::Trusted
            } else {
                CertificateTrustStatus::Pending
            },
            requires_elevation: !trusted,
            elevation: (!trusted).then(|| certificate_trust_elevation_request(&ca_path)),
            status_message: if trusted {
                "Local certificate authority is trusted by the current user trust store."
                    .to_string()
            } else {
                "Local certificate authority is generated but not trusted yet.".to_string()
            },
        })
    }

    fn trust_local_certificate_authority(&self) -> AppResult<CertificateTrustResult> {
        let ca_path = self.certificate_authority_path();

        if !ca_path.exists() {
            return Ok(CertificateTrustResult {
                certificate_authority_path: ca_path.to_string_lossy().into_owned(),
                status: CertificateTrustStatus::Missing,
                requires_elevation: false,
                elevation: None,
                status_message: "Generate the local certificate authority before trusting it."
                    .to_string(),
            });
        }

        let Some((runner, program_path, args)) = trust_command(&ca_path)? else {
            return Ok(CertificateTrustResult {
                certificate_authority_path: ca_path.to_string_lossy().into_owned(),
                status: CertificateTrustStatus::Pending,
                requires_elevation: true,
                elevation: Some(certificate_trust_elevation_request(&ca_path)),
                status_message:
                    "Certificate trust management is not automated for this operating system yet."
                        .to_string(),
            });
        };

        match run_trust_command(&runner, &program_path, args) {
            Ok(_) => Ok(CertificateTrustResult {
                certificate_authority_path: ca_path.to_string_lossy().into_owned(),
                status: CertificateTrustStatus::Trusted,
                requires_elevation: false,
                elevation: None,
                status_message: "Local certificate authority trust request completed.".to_string(),
            }),
            Err(error) => Ok(CertificateTrustResult {
                certificate_authority_path: ca_path.to_string_lossy().into_owned(),
                status: CertificateTrustStatus::Pending,
                requires_elevation: true,
                elevation: Some(certificate_trust_elevation_request(&ca_path)),
                status_message: format!("Certificate trust requires user approval: {error}"),
            }),
        }
    }
}

fn openssl_domain_config(domain: &str) -> String {
    format!(
        "[req]\ndistinguished_name = req_distinguished_name\nreq_extensions = req_ext\nprompt = no\n\n[req_distinguished_name]\nCN = {domain}\n\n[req_ext]\nsubjectAltName = @alt_names\nkeyUsage = critical, digitalSignature, keyEncipherment\nextendedKeyUsage = serverAuth\n\n[alt_names]\nDNS.1 = {domain}\n"
    )
}

fn run_openssl(
    runner: &CommandRunner,
    openssl_path: &Path,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> AppResult<ProcessOutput> {
    let output = runner.execute(
        ProcessCommand::new(openssl_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(CERTIFICATE_TIMEOUT),
    )?;

    ensure_successful_output("OpenSSL", output)
}

fn run_trust_command(
    runner: &CommandRunner,
    program_path: &Path,
    args: Vec<String>,
) -> AppResult<ProcessOutput> {
    let output = runner.execute(
        ProcessCommand::new(program_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(CERTIFICATE_TIMEOUT),
    )?;

    ensure_successful_output("certificate trust", output)
}

fn ensure_successful_output(context: &str, output: ProcessOutput) -> AppResult<ProcessOutput> {
    if output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "{context} command timed out"
        )));
    }

    if output.exit_code != Some(0) {
        return Err(AppError::Infrastructure(format!(
            "{context} command failed: {}",
            if output.stderr.trim().is_empty() {
                output.stdout.trim()
            } else {
                output.stderr.trim()
            }
        )));
    }

    Ok(output)
}

fn certificate_authority_is_trusted(ca_path: &Path) -> AppResult<bool> {
    if cfg!(target_os = "macos") {
        let Some(security_path) = ExecutableResolver::from_env().resolve("security") else {
            return Ok(false);
        };
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([security_path.clone()])
                .with_default_timeout(CERTIFICATE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );
        let output = runner.execute(
            ProcessCommand::new(security_path.to_string_lossy().into_owned())
                .args(["verify-cert", "-c", &ca_path.to_string_lossy()])
                .timeout(CERTIFICATE_TIMEOUT),
        )?;

        return Ok(output.exit_code == Some(0) && !output.timed_out);
    }

    if cfg!(windows) {
        let Some(certutil_path) = ExecutableResolver::from_env().resolve("certutil") else {
            return Ok(false);
        };
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([certutil_path.clone()])
                .with_default_timeout(CERTIFICATE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );
        let output = runner.execute(
            ProcessCommand::new(certutil_path.to_string_lossy().into_owned())
                .args(["-user", "-store", "Root"])
                .timeout(CERTIFICATE_TIMEOUT),
        )?;

        return Ok(output.stdout.contains(CA_COMMON_NAME));
    }

    Ok(false)
}

fn trust_command(ca_path: &Path) -> AppResult<Option<(CommandRunner, PathBuf, Vec<String>)>> {
    if cfg!(target_os = "macos") {
        let Some(security_path) = ExecutableResolver::from_env().resolve("security") else {
            return Ok(None);
        };
        let keychain_path = BaseDirs::new()
            .map(|dirs| {
                dirs.home_dir()
                    .join("Library")
                    .join("Keychains")
                    .join("login.keychain-db")
            })
            .ok_or_else(|| {
                AppError::Configuration("failed to resolve login keychain path".to_string())
            })?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([security_path.clone()])
                .with_default_timeout(CERTIFICATE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        return Ok(Some((
            runner,
            security_path,
            vec![
                "add-trusted-cert".to_string(),
                "-d".to_string(),
                "-r".to_string(),
                "trustRoot".to_string(),
                "-k".to_string(),
                keychain_path.to_string_lossy().into_owned(),
                ca_path.to_string_lossy().into_owned(),
            ],
        )));
    }

    if cfg!(windows) {
        let Some(certutil_path) = ExecutableResolver::from_env().resolve("certutil") else {
            return Ok(None);
        };
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([certutil_path.clone()])
                .with_default_timeout(CERTIFICATE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        return Ok(Some((
            runner,
            certutil_path,
            vec![
                "-user".to_string(),
                "-addstore".to_string(),
                "Root".to_string(),
                ca_path.to_string_lossy().into_owned(),
            ],
        )));
    }

    Ok(None)
}

fn certificate_trust_elevation_request(ca_path: &Path) -> PermissionElevationRequest {
    let command_preview = if cfg!(target_os = "macos") {
        vec![format!(
            "security add-trusted-cert -d -r trustRoot -k ~/Library/Keychains/login.keychain-db \"{}\"",
            ca_path.to_string_lossy()
        )]
    } else if cfg!(windows) {
        vec![format!(
            "certutil -user -addstore Root \"{}\"",
            ca_path.to_string_lossy()
        )]
    } else {
        vec![
            "Trust the generated root CA using the operating system certificate trust UI."
                .to_string(),
        ]
    };

    PermissionElevationRequest {
        kind: PermissionElevationKind::CertificateTrust,
        title: "Certificate trust approval required".to_string(),
        reason: "Trusting a local root certificate authority changes the user trust store and requires explicit approval.".to_string(),
        command_preview,
        requires_admin: cfg!(target_os = "macos"),
        status_message: "Only trust this certificate authority for local AxiomPHP development domains.".to_string(),
    }
}

#[cfg(unix)]
fn lock_private_directory(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to lock certificate directory permissions: {error}"
        ))
    })
}

#[cfg(not(unix))]
fn lock_private_directory(_path: &Path) -> AppResult<()> {
    Ok(())
}

#[cfg(unix)]
fn lock_private_file(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to lock certificate file permissions: {error}"
        ))
    })
}

#[cfg(not(unix))]
fn lock_private_file(_path: &Path) -> AppResult<()> {
    Ok(())
}
