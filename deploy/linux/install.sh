#!/bin/sh
#
# CognitiveOS Personal Linux bootstrap template.
#
# Release automation renders every CognitiveOS policy placeholder below into a reviewed,
# version-specific script before publication. The source template deliberately
# fails closed and must not be represented as a usable release installer.

set -eu
umask 077

RELEASE_VERSION="@COGNITIVEOS_RELEASE_VERSION@"
RELEASE_OBJECT_DIRECTORY="@COGNITIVEOS_RELEASE_OBJECT_DIRECTORY@"
ALLOWED_REDIRECT_HOST="@COGNITIVEOS_ALLOWED_REDIRECT_HOST@"
INSTALLER_SHA256="@COGNITIVEOS_INSTALLER_SHA256@"
TRUSTED_KEYRING_VERSION="@COGNITIVEOS_TRUSTED_KEYRING_VERSION@"
TRUSTED_KEY_ID="@COGNITIVEOS_TRUSTED_KEY_ID@"
TRUSTED_PUBLIC_KEY_BASE64URL="@COGNITIVEOS_TRUSTED_PUBLIC_KEY_BASE64URL@"
EXPECTED_PI_VERSION="@COGNITIVEOS_EXPECTED_PI_VERSION@"
EXPECTED_PI_INTEGRITY="@COGNITIVEOS_EXPECTED_PI_INTEGRITY@"

INSTALLER_FILENAME="cognitiveos-linux-bundle-installer"
MANIFEST_FILENAME="manifest.json"
ARTIFACT_FILENAME="cognitiveos-linux-x86_64.tar.gz"
STATEMENT_FILENAME="attestation.statement.json"
SIGNATURE_FILENAME="attestation.signature.json"
MAX_INSTALLER_BYTES=33554432
MAX_METADATA_BYTES=65536
MAX_ARTIFACT_BYTES=536870912
CONNECT_TIMEOUT_SECONDS=10
TRANSFER_TIMEOUT_SECONDS=120
RETRY_COUNT=2

TEMP_DIRECTORY=""
TEMP_OWNER_MARKER=""

print_error() {
    printf '%s\n' "CognitiveOS Linux bootstrap failed: $1" >&2
}

is_unrendered_template() {
    case "$1" in
        *"@COGNITIVEOS_"*"@"*) return 0 ;;
        *) return 1 ;;
    esac
}

has_control_character() {
    sanitized_value=$(printf '%s' "$1" | tr -d '\011\012\015')
    [ "$sanitized_value" != "$1" ]
}

require_rendered_value() {
    if [ -z "$1" ] || is_unrendered_template "$1" || has_control_character "$1"; then
        print_error "release policy is not rendered"
        exit 64
    fi
}

validate_version() {
    case "$RELEASE_VERSION" in
        *[!A-Za-z0-9._-]*|"")
            print_error "release policy version is invalid"
            exit 64
            ;;
    esac
}

validate_release_policy() {
    require_rendered_value "$RELEASE_VERSION"
    require_rendered_value "$RELEASE_OBJECT_DIRECTORY"
    require_rendered_value "$ALLOWED_REDIRECT_HOST"
    require_rendered_value "$INSTALLER_SHA256"
    require_rendered_value "$TRUSTED_KEYRING_VERSION"
    require_rendered_value "$TRUSTED_KEY_ID"
    require_rendered_value "$TRUSTED_PUBLIC_KEY_BASE64URL"
    require_rendered_value "$EXPECTED_PI_VERSION"
    require_rendered_value "$EXPECTED_PI_INTEGRITY"
    validate_version

    case "$RELEASE_OBJECT_DIRECTORY" in
        https://*/*) ;;
        *) print_error "release policy URL is invalid"; exit 64 ;;
    esac
    case "$RELEASE_OBJECT_DIRECTORY" in
        *"@"*|*"?"*|*"#"*) print_error "release policy URL is invalid"; exit 64 ;;
    esac
    case "$ALLOWED_REDIRECT_HOST" in
        *[!A-Za-z0-9.-]*|""|.*|*..*) print_error "redirect host policy is invalid"; exit 64 ;;
    esac
    case "$INSTALLER_SHA256" in
        sha256:*) installer_digest_hex=${INSTALLER_SHA256#sha256:} ;;
        *) print_error "installer digest policy is invalid"; exit 64 ;;
    esac
    if [ "${#installer_digest_hex}" -ne 64 ]; then
        print_error "installer digest policy is invalid"
        exit 64
    fi
    case "$installer_digest_hex" in
        *[!0123456789abcdef]*) print_error "installer digest policy is invalid"; exit 64 ;;
    esac
}

create_private_temporary_directory() {
    temporary_base_directory=${TMPDIR:-/tmp}
    if [ ! -d "$temporary_base_directory" ]; then
        print_error "temporary directory is unavailable"
        exit 70
    fi
    TEMP_DIRECTORY=$(mktemp -d "${temporary_base_directory}/cognitiveos-bootstrap.XXXXXXXX") || {
        print_error "private temporary directory could not be created"
        exit 70
    }
    TEMP_OWNER_MARKER="${TEMP_DIRECTORY}/.cognitiveos-bootstrap-owner"
    printf '%s\n' "owned" > "$TEMP_OWNER_MARKER"
}

cleanup_temporary_directory() {
    exit_status=$?
    trap - EXIT HUP INT TERM
    if [ -n "$TEMP_DIRECTORY" ] && [ -n "$TEMP_OWNER_MARKER" ] \
        && [ -f "$TEMP_OWNER_MARKER" ]; then
        case "$TEMP_DIRECTORY" in
            "${TMPDIR:-/tmp}"/cognitiveos-bootstrap.*)
                rm -rf -- "$TEMP_DIRECTORY"
                ;;
        esac
    fi
    exit "$exit_status"
}

download_once() {
    download_url=$1
    partial_path=$2
    header_path=$3
    maximum_bytes=$4
    curl --disable --silent --show-error --fail --globoff --proto '=https' \
        --connect-timeout "$CONNECT_TIMEOUT_SECONDS" \
        --max-time "$TRANSFER_TIMEOUT_SECONDS" \
        --retry "$RETRY_COUNT" --retry-delay 1 --max-filesize "$maximum_bytes" \
        --dump-header "$header_path" --output "$partial_path" \
        --write-out '%{http_code}' --url "$download_url"
}

redirect_location() {
    awk 'BEGIN { IGNORECASE = 1 } /^Location:[[:space:]]*/ { sub(/^[^:]*:[[:space:]]*/, ""); sub(/\r$/, ""); location = $0 } END { print location }' "$1"
}

is_allowed_redirect() {
    redirect_url=$1
    case "$redirect_url" in
        "https://${ALLOWED_REDIRECT_HOST}/"*) return 0 ;;
        *) return 1 ;;
    esac
}

download_file() {
    object_filename=$1
    maximum_bytes=$2
    final_path="${BUNDLE_DIRECTORY}/${object_filename}"
    partial_path="${final_path}.partial"
    header_path="${TEMP_DIRECTORY}/${object_filename}.headers"
    initial_url="${RELEASE_OBJECT_DIRECTORY}/${object_filename}"

    http_status=$(download_once "$initial_url" "$partial_path" "$header_path" "$maximum_bytes") || {
        print_error "download failed"
        exit 69
    }
    if [ "$http_status" = "200" ]; then
        mv -f -- "$partial_path" "$final_path"
        return 0
    fi

    case "$http_status" in
        301|302|303|307|308) ;;
        *) print_error "download returned an unsupported response"; exit 69 ;;
    esac
    redirect_url=$(redirect_location "$header_path")
    if [ -z "$redirect_url" ] || ! is_allowed_redirect "$redirect_url"; then
        print_error "download redirect is not allowed"
        exit 69
    fi

    rm -f -- "$partial_path" "$header_path"
    redirected_status=$(download_once "$redirect_url" "$partial_path" "$header_path" "$maximum_bytes") || {
        print_error "redirected download failed"
        exit 69
    }
    if [ "$redirected_status" != "200" ]; then
        print_error "redirected download did not complete"
        exit 69
    fi
    mv -f -- "$partial_path" "$final_path"
}

verify_installer_digest() {
    actual_digest=$(sha256sum "$INSTALLER_PATH" | awk '{print $1}') || {
        print_error "bootstrap installer digest could not be computed"
        exit 69
    }
    if [ "sha256:${actual_digest}" != "$INSTALLER_SHA256" ]; then
        print_error "bootstrap installer digest does not match release policy"
        exit 69
    fi
    chmod 0700 "$INSTALLER_PATH"
}

run_local_installer() {
    "$INSTALLER_PATH" \
        --bundle-directory "$BUNDLE_DIRECTORY" \
        --expected-release-version "$RELEASE_VERSION" \
        --expected-pi-version "$EXPECTED_PI_VERSION" \
        --expected-pi-integrity "$EXPECTED_PI_INTEGRITY" \
        --keyring-version "$TRUSTED_KEYRING_VERSION" \
        --key-id "$TRUSTED_KEY_ID" \
        --public-key-base64url "$TRUSTED_PUBLIC_KEY_BASE64URL"
}

main() {
    if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
        print_error "this bootstrap supports Linux x86_64 only"
        exit 64
    fi
    if [ "$#" -gt 1 ] || { [ "$#" -eq 1 ] && [ "$1" != "$RELEASE_VERSION" ]; }; then
        print_error "requested version does not match inspected release policy"
        exit 64
    fi

    validate_release_policy
    create_private_temporary_directory
    BUNDLE_DIRECTORY="${TEMP_DIRECTORY}/bundle"
    INSTALLER_PATH="${TEMP_DIRECTORY}/${INSTALLER_FILENAME}"
    mkdir -p "$BUNDLE_DIRECTORY"

    download_file "$INSTALLER_FILENAME" "$MAX_INSTALLER_BYTES"
    mv -f -- "${BUNDLE_DIRECTORY}/${INSTALLER_FILENAME}" "$INSTALLER_PATH"
    verify_installer_digest
    download_file "$MANIFEST_FILENAME" "$MAX_METADATA_BYTES"
    download_file "$STATEMENT_FILENAME" "$MAX_METADATA_BYTES"
    download_file "$SIGNATURE_FILENAME" "$MAX_METADATA_BYTES"
    download_file "$ARTIFACT_FILENAME" "$MAX_ARTIFACT_BYTES"
    run_local_installer
}

trap cleanup_temporary_directory EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
main "$@"
