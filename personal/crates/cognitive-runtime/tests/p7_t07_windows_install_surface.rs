//! P7-T07 coverage for the inspected Windows install surface: the bootstrap
//! template and the per-user scheduled-task template.
//!
//! Static structural checks run on every platform; behavioral fail-closed
//! negatives execute the real template through the absolute system Windows
//! PowerShell on `CI-WINDOWS-MSVC-01`. Nothing here claims B01-W, Windows
//! install parity, a Gate, release, or Profile.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("runtime crate must be nested under repository root")
        .to_path_buf()
}

fn bootstrap_template() -> String {
    std::fs::read_to_string(repository_root().join("personal/deploy/windows/install.ps1"))
        .expect("windows bootstrap template must exist")
}

fn task_template() -> String {
    std::fs::read_to_string(repository_root().join("personal/deploy/windows/cognitiveos-personal-task.xml"))
        .expect("windows scheduled task template must exist")
}

#[test]
fn bootstrap_template_contains_only_inspected_fail_closed_primitives() {
    let template = bootstrap_template();

    for required_fragment in [
        "Set-StrictMode -Version 2.0",
        "$ErrorActionPreference = 'Stop'",
        "@COGNITIVEOS_RELEASE_VERSION@",
        "@COGNITIVEOS_INSTALLER_SHA256@",
        "'System32\\curl.exe'",
        "--proto",
        "'=https'",
        "--max-filesize",
        ".partial",
        "cognitiveos-bootstrap.",
        "release policy is not rendered",
        "cognitiveos-windows-bundle-installer.exe",
        "Get-FileHash",
        "Remove-TemporaryDirectory",
        "sha256:[0-9a-f]{64}",
    ] {
        assert!(
            template.contains(required_fragment),
            "windows bootstrap template is missing {required_fragment:?}"
        );
    }

    for forbidden_fragment in [
        "Invoke-Expression",
        "iex ",
        "DownloadString",
        "Invoke-WebRequest",
        "Invoke-RestMethod",
        "Set-ExecutionPolicy",
        "ConvertTo-SecureString",
        "ConvertFrom-SecureString",
        "SecretRef",
        "schtasks",
        "Start-Process -Verb",
        "RunAs",
        "sudo ",
        "SigningKey",
        "TEST_ONLY_PRIVATE",
        "npm install",
        "pnpm install",
    ] {
        assert!(
            !template.contains(forbidden_fragment),
            "windows bootstrap template contains forbidden {forbidden_fragment:?}"
        );
    }
}

#[test]
fn task_template_is_least_privilege_interactive_and_unrendered() {
    let template = task_template();

    for required_fragment in [
        "intentionally unrendered",
        "<LogonTrigger>",
        "<LogonType>InteractiveToken</LogonType>",
        "<RunLevel>LeastPrivilege</RunLevel>",
        "@COGNITIVEOS_RELEASE_ROOT@\\bin\\kernel-server.exe",
        "--personal --bind 127.0.0.1:@COGNITIVEOS_PERSONAL_HEALTH_PORT@ --runtime-root @COGNITIVEOS_RUNTIME_ROOT@",
    ] {
        assert!(
            template.contains(required_fragment),
            "windows task template is missing {required_fragment:?}"
        );
    }

    for forbidden_fragment in [
        "HighestAvailable",
        "<Password>",
        "S4U",
        "S-1-5-18",
        "NT AUTHORITY",
    ] {
        assert!(
            !template.contains(forbidden_fragment),
            "windows task template contains forbidden {forbidden_fragment:?}"
        );
    }
}

#[test]
fn task_template_reuses_the_linux_service_placeholder_contract() {
    let linux_unit = std::fs::read_to_string(
        repository_root().join("personal/deploy/linux/cognitiveos-personal.service"),
    )
    .expect("linux service template must exist");
    let windows_task = task_template();

    // The Windows daemon start must bind the same rendered facts as the Linux
    // user service so release rendering stays a single policy surface.
    for placeholder in [
        "@COGNITIVEOS_RELEASE_ROOT@",
        "@COGNITIVEOS_PERSONAL_HEALTH_PORT@",
        "@COGNITIVEOS_RUNTIME_ROOT@",
    ] {
        assert!(
            linux_unit.contains(placeholder),
            "linux unit lost {placeholder:?}; the shared contract changed"
        );
        assert!(
            windows_task.contains(placeholder),
            "windows task template is missing {placeholder:?}"
        );
    }
    assert!(windows_task.contains("--personal"));
    assert!(linux_unit.contains("--personal"));
}

#[cfg(target_os = "windows")]
mod windows_native {
    use super::{bootstrap_template, repository_root};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    fn system_powershell() -> PathBuf {
        let system_root = std::env::var_os("SystemRoot").expect("SystemRoot must exist");
        Path::new(&system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe")
    }

    struct BootstrapFixture {
        _temporary_directory: tempfile::TempDir,
        observed_temp: PathBuf,
        script_directory: PathBuf,
    }

    impl BootstrapFixture {
        fn new() -> Self {
            let temporary_directory = tempfile::tempdir().unwrap();
            let observed_temp = temporary_directory.path().join("observed-temp");
            let script_directory = temporary_directory.path().join("scripts");
            fs::create_dir_all(&observed_temp).unwrap();
            fs::create_dir_all(&script_directory).unwrap();
            Self {
                _temporary_directory: temporary_directory,
                observed_temp,
                script_directory,
            }
        }

        fn run(&self, script_path: &Path, arguments: &[&str]) -> Output {
            let mut command = Command::new(system_powershell());
            command
                .arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-File")
                .arg(script_path);
            for argument in arguments {
                command.arg(argument);
            }
            command
                .env("TEMP", &self.observed_temp)
                .env("TMP", &self.observed_temp)
                .output()
                .unwrap()
        }

        fn observed_temp_entries(&self) -> Vec<PathBuf> {
            fs::read_dir(&self.observed_temp)
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        }

        fn write_rendered(&self, rendered: &str) -> PathBuf {
            let path = self.script_directory.join("install-rendered.ps1");
            fs::write(&path, rendered).unwrap();
            path
        }
    }

    fn render_template(version: &str, object_directory: &str, installer_digest: &str) -> String {
        let replacements = [
            ("@COGNITIVEOS_RELEASE_VERSION@", version),
            ("@COGNITIVEOS_RELEASE_OBJECT_DIRECTORY@", object_directory),
            (
                "@COGNITIVEOS_ALLOWED_REDIRECT_HOST@",
                "redirect.example.test",
            ),
            ("@COGNITIVEOS_INSTALLER_SHA256@", installer_digest),
            (
                "@COGNITIVEOS_TRUSTED_KEYRING_VERSION@",
                "p7t07-test-keyring-v1",
            ),
            ("@COGNITIVEOS_TRUSTED_KEY_ID@", "p7t07-test-key"),
            (
                "@COGNITIVEOS_TRUSTED_PUBLIC_KEY_BASE64URL@",
                "p7t07-test-public-key",
            ),
            ("@COGNITIVEOS_EXPECTED_PI_VERSION@", "0.81.1"),
            (
                "@COGNITIVEOS_EXPECTED_PI_INTEGRITY@",
                "sha512:pinned-pi-integrity",
            ),
        ];
        replacements
            .into_iter()
            .fold(bootstrap_template(), |rendered, (needle, replacement)| {
                rendered.replace(needle, replacement)
            })
    }

    const VALID_TEST_DIGEST: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn stderr(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    #[test]
    fn unrendered_template_rejects_before_any_network_or_temp_side_effect() {
        let fixture = BootstrapFixture::new();
        let output = fixture.run(&repository_root().join("personal/deploy/windows/install.ps1"), &[]);

        assert_eq!(
            output.status.code(),
            Some(64),
            "stderr: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("release policy is not rendered"));
        assert!(fixture.observed_temp_entries().is_empty());
    }

    #[test]
    fn rendered_bootstrap_rejects_a_mismatched_requested_version_before_any_download() {
        let fixture = BootstrapFixture::new();
        let rendered = render_template(
            "1.2.3",
            "https://releases.example.test/v1.2.3",
            VALID_TEST_DIGEST,
        );
        let script = fixture.write_rendered(&rendered);
        let output = fixture.run(&script, &["9.9.9"]);

        assert_eq!(
            output.status.code(),
            Some(64),
            "stderr: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains("requested version does not match inspected release policy")
        );
        assert!(fixture.observed_temp_entries().is_empty());
    }

    #[test]
    fn rendered_bootstrap_rejects_a_malformed_installer_digest_before_any_download() {
        let fixture = BootstrapFixture::new();
        let rendered = render_template(
            "1.2.3",
            "https://releases.example.test/v1.2.3",
            "sha256:not-a-valid-digest",
        );
        let script = fixture.write_rendered(&rendered);
        let output = fixture.run(&script, &[]);

        assert_eq!(
            output.status.code(),
            Some(64),
            "stderr: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("installer digest policy is invalid"));
        assert!(fixture.observed_temp_entries().is_empty());
    }

    #[test]
    fn rendered_bootstrap_rejects_a_non_https_object_directory_before_any_download() {
        let fixture = BootstrapFixture::new();
        let rendered = render_template(
            "1.2.3",
            "http://releases.example.test/v1.2.3",
            VALID_TEST_DIGEST,
        );
        let script = fixture.write_rendered(&rendered);
        let output = fixture.run(&script, &[]);

        assert_eq!(
            output.status.code(),
            Some(64),
            "stderr: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("release policy URL is invalid"));
        assert!(fixture.observed_temp_entries().is_empty());
    }

    #[test]
    fn rendered_bootstrap_rejects_extra_arguments_before_any_download() {
        let fixture = BootstrapFixture::new();
        let rendered = render_template(
            "1.2.3",
            "https://releases.example.test/v1.2.3",
            VALID_TEST_DIGEST,
        );
        let script = fixture.write_rendered(&rendered);
        let output = fixture.run(&script, &["1.2.3", "unexpected-extra"]);

        assert_eq!(
            output.status.code(),
            Some(64),
            "stderr: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("unsupported extra arguments"));
        assert!(fixture.observed_temp_entries().is_empty());
    }

    #[test]
    fn task_template_parses_as_least_privilege_interactive_task_xml() {
        let fixture = BootstrapFixture::new();
        let task_path = repository_root().join("personal/deploy/windows/cognitiveos-personal-task.xml");
        let check_script = fixture.script_directory.join("check-task-xml.ps1");
        fs::write(
            &check_script,
            format!(
                "$ErrorActionPreference = 'Stop'\n\
                 try {{ $document = [xml](Get-Content -Raw -LiteralPath '{}') }} catch {{ exit 2 }}\n\
                 if ($document.Task.Principals.Principal.LogonType -cne 'InteractiveToken') {{ exit 3 }}\n\
                 if ($document.Task.Principals.Principal.RunLevel -cne 'LeastPrivilege') {{ exit 4 }}\n\
                 if ($document.Task.Actions.Exec.Command -notlike '*kernel-server.exe') {{ exit 5 }}\n\
                 exit 0\n",
                task_path.display()
            ),
        )
        .unwrap();
        let output = fixture.run(&check_script, &[]);
        assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    }
}
