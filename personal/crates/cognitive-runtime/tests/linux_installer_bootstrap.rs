//! Process coverage for the inspected Linux bootstrap boundary.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("runtime crate must be nested under repository root")
        .to_path_buf()
}

#[test]
fn production_template_contains_only_inspected_fail_closed_bootstrap_primitives() {
    let template =
        std::fs::read_to_string(repository_root().join("personal/deploy/linux/install.sh"))
            .expect("production bootstrap template must exist");

    for required_fragment in [
        "#!/bin/sh",
        "set -eu",
        "umask 077",
        "curl --disable",
        "mktemp -d",
        "trap cleanup_temporary_directory EXIT",
        "run_local_installer",
        "@COGNITIVEOS_RELEASE_VERSION@",
    ] {
        assert!(
            template.contains(required_fragment),
            "bootstrap template is missing {required_fragment:?}"
        );
    }
    for forbidden_fragment in [
        "curl | sh",
        "eval ",
        "sudo ",
        "systemctl",
        "install_linux_bundle",
        "SigningKey",
        "TEST_ONLY_PRIVATE",
        "npm install",
        "pnpm install",
    ] {
        assert!(
            !template.contains(forbidden_fragment),
            "bootstrap template contains forbidden {forbidden_fragment:?}"
        );
    }
}

#[cfg(unix)]
mod unix {
    use super::repository_root;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use cognitive_runtime::LinuxBundleManifest;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    const TEST_KEY_ID: &str = "p1t08-bootstrap-test-key";
    const TEST_KEYRING_VERSION: &str = "p1t08-bootstrap-test-keyring-v1";
    const PI_VERSION: &str = "0.81.1";
    const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
    const ARTIFACT_FILENAME: &str = "cognitiveos-linux-x86_64.tar.gz";

    struct BootstrapFixture {
        temporary_directory: tempfile::TempDir,
        release_directory: PathBuf,
        rendered_installer: PathBuf,
        fake_curl_directory: PathBuf,
        curl_log: PathBuf,
        installer_log: PathBuf,
        temporary_base: PathBuf,
        public_key: String,
        installer_digest: String,
    }

    impl BootstrapFixture {
        fn new() -> Self {
            let temporary_directory = tempfile::tempdir().unwrap();
            let release_directory = temporary_directory.path().join("release");
            let fake_curl_directory = temporary_directory.path().join("fake-bin");
            let temporary_base = temporary_directory.path().join("private-temp");
            fs::create_dir_all(&release_directory).unwrap();
            fs::create_dir_all(&fake_curl_directory).unwrap();
            fs::create_dir_all(&temporary_base).unwrap();

            let signing_key = SigningKey::from_bytes(&[0x71; 32]);
            let public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
            write_signed_bundle(&release_directory, &signing_key);

            let installer_wrapper = release_directory.join("cognitiveos-linux-bundle-installer");
            fs::write(
                &installer_wrapper,
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$COGNITIVEOS_TEST_VERIFIER_LOG\"\nbundle_directory=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = --bundle-directory ]; then bundle_directory=$2; break; fi\n  shift\ndone\nexec \"$COGNITIVEOS_TEST_REAL_VERIFIER\" --bundle-directory \"$bundle_directory\" --expected-pi-version \"$COGNITIVEOS_TEST_PI_VERSION\" --expected-pi-integrity \"$COGNITIVEOS_TEST_PI_INTEGRITY\" --keyring-version \"$COGNITIVEOS_TEST_KEYRING_VERSION\" --key-id \"$COGNITIVEOS_TEST_KEY_ID\" --public-key-base64url \"$COGNITIVEOS_TEST_PUBLIC_KEY\"\n",
            )
            .unwrap();
            fs::set_permissions(&installer_wrapper, fs::Permissions::from_mode(0o700)).unwrap();
            let installer_digest = sha256_file(&installer_wrapper);

            let rendered_installer = temporary_directory.path().join("install.sh");
            let template =
                fs::read_to_string(repository_root().join("personal/deploy/linux/install.sh"))
                    .unwrap();
            let rendered = render_template(&template, &public_key, &installer_digest);
            fs::write(&rendered_installer, rendered).unwrap();
            fs::set_permissions(&rendered_installer, fs::Permissions::from_mode(0o700)).unwrap();

            let curl_log = temporary_directory.path().join("curl.log");
            let installer_log = temporary_directory.path().join("installer.log");
            write_fake_curl(&fake_curl_directory.join("curl"));

            Self {
                temporary_directory,
                release_directory,
                rendered_installer,
                fake_curl_directory,
                curl_log,
                installer_log,
                temporary_base,
                public_key,
                installer_digest,
            }
        }

        fn run(&self, mode: &str) -> Output {
            Command::new("sh")
                .arg(&self.rendered_installer)
                .env(
                    "PATH",
                    format!("{}:/usr/bin:/bin", self.fake_curl_directory.display()),
                )
                .env("TMPDIR", &self.temporary_base)
                .env(
                    "COGNITIVEOS_TEST_RELEASE_DIRECTORY",
                    &self.release_directory,
                )
                .env("COGNITIVEOS_TEST_CURL_LOG", &self.curl_log)
                .env("COGNITIVEOS_TEST_VERIFIER_LOG", &self.installer_log)
                .env("COGNITIVEOS_TEST_PI_VERSION", PI_VERSION)
                .env("COGNITIVEOS_TEST_PI_INTEGRITY", PI_INTEGRITY)
                .env("COGNITIVEOS_TEST_KEYRING_VERSION", TEST_KEYRING_VERSION)
                .env("COGNITIVEOS_TEST_KEY_ID", TEST_KEY_ID)
                .env("COGNITIVEOS_TEST_PUBLIC_KEY", &self.public_key)
                .env(
                    "COGNITIVEOS_TEST_REAL_VERIFIER",
                    env!("CARGO_BIN_EXE_linux_bundle_verifier"),
                )
                .env("COGNITIVEOS_TEST_CURL_MODE", mode)
                .output()
                .unwrap()
        }

        fn bootstrap_directories(&self) -> Vec<PathBuf> {
            fs::read_dir(&self.temporary_base)
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        }
    }

    #[test]
    fn rendered_bootstrap_binds_the_inspected_release_version_at_installer_handoff() {
        let fixture = BootstrapFixture::new();
        let output = fixture.run("success");

        assert!(output.status.success(), "stderr: {}", stderr(&output));
        let installer_invocations = fs::read_to_string(&fixture.installer_log).unwrap();
        assert_eq!(installer_invocations.lines().count(), 1);
        assert!(installer_invocations.contains("--bundle-directory"));
        assert!(installer_invocations.contains("--expected-release-version 1.2.3"));
        assert!(installer_invocations.contains("cognitiveos-bootstrap."));
        assert!(stdout(&output).contains("verified-linux-bundle version=1.2.3"));
        assert!(!stdout(&output).contains("p1t08-bootstrap-artifact"));
        assert!(fixture.bootstrap_directories().is_empty());

        let curl_log = fs::read_to_string(&fixture.curl_log).unwrap();
        assert!(curl_log.contains("--disable"));
        assert!(curl_log.contains(".partial"));
        assert!(!curl_log.contains(&fixture.public_key));
    }

    #[test]
    fn production_installer_rejects_a_valid_bundle_with_a_mismatched_inspected_version_before_xdg_mutation()
     {
        let fixture = BootstrapFixture::new();
        let xdg_data_home = fixture.temporary_directory.path().join("xdg-data");
        let output = Command::new(env!("CARGO_BIN_EXE_linux_bundle_installer"))
            .args([
                "--bundle-directory",
                fixture.release_directory.to_str().unwrap(),
                "--expected-release-version",
                "9.9.9",
                "--expected-pi-version",
                PI_VERSION,
                "--expected-pi-integrity",
                PI_INTEGRITY,
                "--keyring-version",
                TEST_KEYRING_VERSION,
                "--key-id",
                TEST_KEY_ID,
                "--public-key-base64url",
                &fixture.public_key,
            ])
            .env("XDG_DATA_HOME", &xdg_data_home)
            .output()
            .unwrap();

        assert!(!output.status.success());
        assert!(stderr(&output).contains("CognitiveOS Linux bundle installation failed"));
        assert!(!xdg_data_home.join("cognitiveos/deployment").exists());
    }

    #[test]
    fn unrendered_template_rejects_before_network_access() {
        let fixture = BootstrapFixture::new();
        let output = Command::new("sh")
            .arg(repository_root().join("personal/deploy/linux/install.sh"))
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", fixture.fake_curl_directory.display()),
            )
            .env("COGNITIVEOS_TEST_CURL_LOG", &fixture.curl_log)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(64));
        assert!(stderr(&output).contains("release policy is not rendered"));
        assert!(!fixture.curl_log.exists());
    }

    #[test]
    fn failed_artifact_download_never_executes_installer_and_cleans_private_downloads() {
        let fixture = BootstrapFixture::new();
        let output = fixture.run("artifact-failure");

        assert!(!output.status.success());
        assert!(!fixture.installer_log.exists());
        assert!(fixture.bootstrap_directories().is_empty());
    }

    #[test]
    fn installer_rejection_preserves_external_deployment_state_and_cleans_downloads() {
        let fixture = BootstrapFixture::new();
        fs::write(
            fixture.release_directory.join(ARTIFACT_FILENAME),
            b"tampered bootstrap artifact",
        )
        .unwrap();
        let deployment_root = fixture.temporary_directory.path().join("deployment");
        fs::create_dir_all(deployment_root.join("user-data")).unwrap();
        fs::write(deployment_root.join("active-version"), "old-version\n").unwrap();
        fs::write(deployment_root.join("user-data/sentinel"), b"user-data").unwrap();

        let output = fixture.run("success");

        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(deployment_root.join("active-version")).unwrap(),
            "old-version\n"
        );
        assert_eq!(
            fs::read(deployment_root.join("user-data/sentinel")).unwrap(),
            b"user-data"
        );
        assert!(fixture.bootstrap_directories().is_empty());
    }

    #[test]
    fn installer_digest_mismatch_prevents_execution() {
        let fixture = BootstrapFixture::new();
        let rendered = render_template(
            &fs::read_to_string(repository_root().join("personal/deploy/linux/install.sh"))
                .unwrap(),
            &fixture.public_key,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        fs::write(&fixture.rendered_installer, rendered).unwrap();

        let output = fixture.run("success");

        assert!(!output.status.success());
        assert!(stderr(&output).contains("bootstrap installer digest does not match"));
        assert!(!fixture.installer_log.exists());
        assert!(fixture.bootstrap_directories().is_empty());
        assert_ne!(
            fixture.installer_digest,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    fn write_signed_bundle(directory: &Path, signing_key: &SigningKey) {
        let artifact = b"p1t08-bootstrap-artifact";
        let manifest = LinuxBundleManifest {
            schema_version: 1,
            product: "cognitiveos-personal".to_owned(),
            platform: "linux-x86_64".to_owned(),
            version: "1.2.3".to_owned(),
            artifact_file: ARTIFACT_FILENAME.to_owned(),
            artifact_sha256: sha256_bytes(artifact),
            attestation_reference: "https://example.invalid/provenance/1.2.3".to_owned(),
            attestation_statement_file: "attestation.statement.json".to_owned(),
            attestation_signature_file: "attestation.signature.json".to_owned(),
            pi_version: PI_VERSION.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        };
        let statement = json!({
            "artifact_file": manifest.artifact_file,
            "artifact_sha256": manifest.artifact_sha256,
            "pi_integrity": manifest.pi_integrity,
            "pi_version": manifest.pi_version,
            "platform": manifest.platform,
            "product": manifest.product,
            "provenance_reference": manifest.attestation_reference,
            "schema": "cognitiveos.personal.linux-bundle-attestation",
            "schema_version": 1,
            "version": manifest.version,
        });
        let statement_bytes = serde_json_canonicalizer::to_vec(&statement).unwrap();
        let signature = signing_key.sign(&statement_bytes);
        fs::write(directory.join(ARTIFACT_FILENAME), artifact).unwrap();
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("attestation.statement.json"),
            statement_bytes,
        )
        .unwrap();
        fs::write(
            directory.join("attestation.signature.json"),
            serde_json::to_vec(&json!({
                "algorithm": "Ed25519",
                "key_id": TEST_KEY_ID,
                "schema": "cognitiveos.personal.linux-bundle-signature",
                "schema_version": 1,
                "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn render_template(template: &str, public_key: &str, installer_digest: &str) -> String {
        let replacements = [
            ("@COGNITIVEOS_RELEASE_VERSION@", "1.2.3"),
            (
                "@COGNITIVEOS_RELEASE_OBJECT_DIRECTORY@",
                "https://releases.example.test/v1.2.3",
            ),
            (
                "@COGNITIVEOS_ALLOWED_REDIRECT_HOST@",
                "redirect.example.test",
            ),
            ("@COGNITIVEOS_INSTALLER_SHA256@", installer_digest),
            (
                "@COGNITIVEOS_TRUSTED_KEYRING_VERSION@",
                TEST_KEYRING_VERSION,
            ),
            ("@COGNITIVEOS_TRUSTED_KEY_ID@", TEST_KEY_ID),
            ("@COGNITIVEOS_TRUSTED_PUBLIC_KEY_BASE64URL@", public_key),
            ("@COGNITIVEOS_EXPECTED_PI_VERSION@", PI_VERSION),
            ("@COGNITIVEOS_EXPECTED_PI_INTEGRITY@", PI_INTEGRITY),
        ];
        replacements
            .into_iter()
            .fold(template.to_owned(), |rendered, (needle, replacement)| {
                rendered.replace(needle, replacement)
            })
    }

    fn write_fake_curl(path: &Path) {
        fs::write(
            path,
            "#!/bin/sh\nset -eu\nprintf '%s\\n' \"$*\" >> \"$COGNITIVEOS_TEST_CURL_LOG\"\noutput=\nheaders=\nurl=\nwhile [ \"$#\" -gt 0 ]; do\n  case \"$1\" in\n    --output) output=$2; shift 2 ;;\n    --dump-header) headers=$2; shift 2 ;;\n    --url) url=$2; shift 2 ;;\n    *) shift ;;\n  esac\ndone\nname=${url##*/}\nif [ \"${COGNITIVEOS_TEST_CURL_MODE:-success}\" = artifact-failure ] && [ \"$name\" = cognitiveos-linux-x86_64.tar.gz ]; then exit 22; fi\nprintf 'HTTP/1.1 200 OK\\r\\n\\r\\n' > \"$headers\"\ncp \"$COGNITIVEOS_TEST_RELEASE_DIRECTORY/$name\" \"$output\"\nprintf 200\n",
        )
        .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn sha256_bytes(bytes: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    fn sha256_file(path: &Path) -> String {
        sha256_bytes(&fs::read(path).unwrap())
    }

    fn stdout(output: &Output) -> String {
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn stderr(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).into_owned()
    }
}
