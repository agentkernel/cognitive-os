//! Bounded OS resource sampling for P9-T04 soak and capacity evidence.
//!
//! The sampler reads only numeric counters for a declared campaign process
//! role. It never opens `cmdline` or `environ`, never resolves a file
//! descriptor to its target, and never attaches to an arbitrary process
//! identity, so a soak observation cannot become a credential, path, or
//! command-line disclosure.

use serde::Serialize;
use std::fs;
use std::path::Path;

/// The only process roles a campaign may sample. An unrecognised process is
/// not sampleable: the campaign never attaches to an arbitrary PID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignProcessRole {
    Daemon,
    PiSidecar,
    AgentAdapter,
}

/// One bounded resource observation. Cumulative counters are stored raw so the
/// report can derive rates without the sampler inventing a unit conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProcessResourceSample {
    pub role: CampaignProcessRole,
    pub pid: u32,
    pub sequence: u64,
    pub cpu_user_ticks: u64,
    pub cpu_system_ticks: u64,
    pub clock_ticks_per_second: u64,
    pub resident_bytes: u64,
    pub thread_count: u64,
    pub file_descriptor_count: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

/// An ordered series for one role and PID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessResourceSeries {
    pub claim_level: &'static str,
    pub role: CampaignProcessRole,
    pub pid: u32,
    pub samples: Vec<ProcessResourceSample>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ResourceSamplerError {
    #[error("process resource facts are incomplete and cannot be sampled")]
    IncompleteProcessFacts,
    #[error("clock tick rate must be a positive value supplied by the caller")]
    InvalidClockTickRate,
    #[error("resource series must be non-empty and strictly ordered by sequence")]
    UnorderedSeries,
    #[error("resource series must observe one declared role and process identity")]
    MixedProcessIdentity,
    #[error("cumulative counters must not decrease; a decrease means the PID was reused")]
    ReusedProcessIdentity,
    #[error("resource observations must remain hypothesis-only")]
    ClaimShapedSeries,
}

const RESOURCE_CLAIM_LEVEL: &str = "hypothesis";

/// Read one bounded sample from a `/proc`-shaped root. Only `stat`, `status`,
/// `io`, and the `fd` entry count are opened.
pub fn read_process_resource_sample(
    role: CampaignProcessRole,
    pid: u32,
    sequence: u64,
    clock_ticks_per_second: u64,
    proc_root: &Path,
) -> Result<ProcessResourceSample, ResourceSamplerError> {
    if clock_ticks_per_second == 0 {
        return Err(ResourceSamplerError::InvalidClockTickRate);
    }
    let process_root = proc_root.join(pid.to_string());
    let stat = read_numeric_file(&process_root.join("stat"))?;
    let status = read_numeric_file(&process_root.join("status"))?;
    let io = read_numeric_file(&process_root.join("io"))?;
    let (cpu_user_ticks, cpu_system_ticks, thread_count) = parse_stat_counters(&stat)?;
    Ok(ProcessResourceSample {
        role,
        pid,
        sequence,
        cpu_user_ticks,
        cpu_system_ticks,
        clock_ticks_per_second,
        resident_bytes: parse_resident_bytes(&status)?,
        thread_count,
        file_descriptor_count: count_file_descriptors(&process_root.join("fd"))?,
        read_bytes: parse_keyed_counter(&io, "read_bytes")?,
        written_bytes: parse_keyed_counter(&io, "write_bytes")?,
    })
}

/// Assemble a validated series from ordered samples.
pub fn build_process_resource_series(
    role: CampaignProcessRole,
    pid: u32,
    samples: Vec<ProcessResourceSample>,
) -> Result<ProcessResourceSeries, ResourceSamplerError> {
    let series = ProcessResourceSeries {
        claim_level: RESOURCE_CLAIM_LEVEL,
        role,
        pid,
        samples,
    };
    validate_process_resource_series(&series)?;
    Ok(series)
}

/// Reject series that mix process identities, reuse a PID, or promote
/// themselves beyond a hypothesis-level observation.
pub fn validate_process_resource_series(
    series: &ProcessResourceSeries,
) -> Result<(), ResourceSamplerError> {
    if series.claim_level != RESOURCE_CLAIM_LEVEL {
        return Err(ResourceSamplerError::ClaimShapedSeries);
    }
    let Some((first, rest)) = series.samples.split_first() else {
        return Err(ResourceSamplerError::UnorderedSeries);
    };
    if first.role != series.role || first.pid != series.pid {
        return Err(ResourceSamplerError::MixedProcessIdentity);
    }
    let mut previous = first;
    for sample in rest {
        if sample.role != series.role || sample.pid != series.pid {
            return Err(ResourceSamplerError::MixedProcessIdentity);
        }
        if sample.sequence <= previous.sequence {
            return Err(ResourceSamplerError::UnorderedSeries);
        }
        if sample.cpu_user_ticks < previous.cpu_user_ticks
            || sample.cpu_system_ticks < previous.cpu_system_ticks
            || sample.read_bytes < previous.read_bytes
            || sample.written_bytes < previous.written_bytes
        {
            return Err(ResourceSamplerError::ReusedProcessIdentity);
        }
        previous = sample;
    }
    Ok(())
}

fn read_numeric_file(path: &Path) -> Result<String, ResourceSamplerError> {
    fs::read_to_string(path).map_err(|_| ResourceSamplerError::IncompleteProcessFacts)
}

/// `utime`, `stime`, and `num_threads` are fields 14, 15, and 20 after the
/// parenthesised process name, which may itself contain spaces.
fn parse_stat_counters(stat: &str) -> Result<(u64, u64, u64), ResourceSamplerError> {
    let after_name = stat
        .rfind(')')
        .and_then(|index| stat.get(index + 1..))
        .ok_or(ResourceSamplerError::IncompleteProcessFacts)?;
    let fields = after_name.split_whitespace().collect::<Vec<_>>();
    let field = |offset: usize| -> Result<u64, ResourceSamplerError> {
        fields
            .get(offset)
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or(ResourceSamplerError::IncompleteProcessFacts)
    };
    Ok((field(11)?, field(12)?, field(17)?))
}

fn parse_resident_bytes(status: &str) -> Result<u64, ResourceSamplerError> {
    let kibibytes = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or(ResourceSamplerError::IncompleteProcessFacts)?;
    kibibytes
        .checked_mul(1024)
        .ok_or(ResourceSamplerError::IncompleteProcessFacts)
}

fn parse_keyed_counter(io: &str, key: &str) -> Result<u64, ResourceSamplerError> {
    io.lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(':'))
        .and_then(|value| value.trim().parse::<u64>().ok())
        .ok_or(ResourceSamplerError::IncompleteProcessFacts)
}

/// Count descriptor entries without resolving any symlink target, so no file
/// path or socket peer enters campaign evidence.
fn count_file_descriptors(descriptor_root: &Path) -> Result<u64, ResourceSamplerError> {
    let entries =
        fs::read_dir(descriptor_root).map_err(|_| ResourceSamplerError::IncompleteProcessFacts)?;
    let mut count = 0_u64;
    for entry in entries {
        entry.map_err(|_| ResourceSamplerError::IncompleteProcessFacts)?;
        count = count.saturating_add(1);
    }
    Ok(count)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs::File;

    const SENTINEL_COMMAND_LINE: &str = "cognitive-daemon--api-key=sk-sentinel-value";
    const SENTINEL_ENVIRONMENT: &str = "DEEPSEEK_API_KEY=sk-sentinel-environment";

    struct ProcFixture {
        root: tempfile::TempDir,
    }

    impl ProcFixture {
        /// A `/proc`-shaped directory that also contains the two files the
        /// sampler must never open.
        fn new(pid: u32, utime: u64, stime: u64, read_bytes: u64, descriptors: usize) -> Self {
            let root = tempfile::tempdir().expect("temporary proc root");
            let process_root = root.path().join(pid.to_string());
            fs::create_dir_all(process_root.join("fd")).expect("fd directory");
            // Fields after the parenthesised name, so index 0 is `state` and
            // `utime`, `stime`, and `num_threads` keep their /proc offsets.
            let mut stat_fields = vec!["0".to_owned(); 40];
            stat_fields[0] = "S".to_owned();
            stat_fields[11] = utime.to_string();
            stat_fields[12] = stime.to_string();
            stat_fields[17] = "9".to_owned();
            fs::write(
                process_root.join("stat"),
                format!("{pid} (kernel server (personal)) {}", stat_fields.join(" ")),
            )
            .expect("stat file");
            fs::write(
                process_root.join("status"),
                "Name:\tkernel-server\nThreads:\t9\nVmRSS:\t   20480 kB\n",
            )
            .expect("status file");
            fs::write(
                process_root.join("io"),
                format!("rchar: 1\nread_bytes: {read_bytes}\nwrite_bytes: 4096\n"),
            )
            .expect("io file");
            fs::write(process_root.join("cmdline"), SENTINEL_COMMAND_LINE).expect("cmdline file");
            fs::write(process_root.join("environ"), SENTINEL_ENVIRONMENT).expect("environ file");
            for descriptor in 0..descriptors {
                File::create(process_root.join("fd").join(descriptor.to_string()))
                    .expect("descriptor entry");
            }
            Self { root }
        }

        fn path(&self) -> &Path {
            self.root.path()
        }
    }

    fn sample_at(fixture: &ProcFixture, pid: u32, sequence: u64) -> ProcessResourceSample {
        read_process_resource_sample(
            CampaignProcessRole::Daemon,
            pid,
            sequence,
            100,
            fixture.path(),
        )
        .expect("sampled process")
    }

    #[test]
    fn sample_records_numeric_counters_only() {
        let fixture = ProcFixture::new(4242, 130, 45, 8_192, 6);
        let sample = sample_at(&fixture, 4242, 1);
        assert_eq!(sample.cpu_user_ticks, 130);
        assert_eq!(sample.cpu_system_ticks, 45);
        assert_eq!(sample.thread_count, 9);
        assert_eq!(sample.resident_bytes, 20_480 * 1024);
        assert_eq!(sample.file_descriptor_count, 6);
        assert_eq!(sample.read_bytes, 8_192);
        assert_eq!(sample.written_bytes, 4_096);
    }

    #[test]
    fn sampler_never_discloses_command_line_or_environment() {
        let fixture = ProcFixture::new(4242, 1, 1, 1, 2);
        let sample = sample_at(&fixture, 4242, 1);
        let serialized = serde_json::to_string(&sample).expect("serialize sample");
        assert!(!serialized.contains("sk-sentinel-value"), "{serialized}");
        assert!(
            !serialized.contains("sk-sentinel-environment"),
            "{serialized}"
        );
        assert!(!serialized.contains("DEEPSEEK"), "{serialized}");
        assert!(!serialized.contains("kernel-server"), "{serialized}");
    }

    #[test]
    fn descriptor_targets_are_counted_but_never_resolved() {
        let fixture = ProcFixture::new(77, 1, 1, 1, 0);
        let descriptor_root = fixture.path().join("77").join("fd");
        File::create(descriptor_root.join("home-wuz-secret-store.sock")).expect("descriptor entry");
        let sample = sample_at(&fixture, 77, 1);
        assert_eq!(sample.file_descriptor_count, 1);
        let serialized = serde_json::to_string(&sample).expect("serialize sample");
        assert!(!serialized.contains("secret-store"), "{serialized}");
    }

    #[test]
    fn missing_or_unreadable_process_facts_fail_closed() {
        let fixture = ProcFixture::new(4242, 1, 1, 1, 1);
        assert_eq!(
            read_process_resource_sample(CampaignProcessRole::Daemon, 9999, 1, 100, fixture.path())
                .unwrap_err(),
            ResourceSamplerError::IncompleteProcessFacts
        );
        fs::write(fixture.path().join("4242").join("status"), "Name:\tx\n")
            .expect("truncated status");
        assert_eq!(
            read_process_resource_sample(CampaignProcessRole::Daemon, 4242, 1, 100, fixture.path())
                .unwrap_err(),
            ResourceSamplerError::IncompleteProcessFacts
        );
    }

    #[test]
    fn zero_clock_tick_rate_fails_before_any_read() {
        let fixture = ProcFixture::new(4242, 1, 1, 1, 1);
        assert_eq!(
            read_process_resource_sample(CampaignProcessRole::Daemon, 4242, 1, 0, fixture.path())
                .unwrap_err(),
            ResourceSamplerError::InvalidClockTickRate
        );
    }

    #[test]
    fn growing_series_is_publishable() {
        let fixture = ProcFixture::new(4242, 10, 2, 1_024, 3);
        let first = sample_at(&fixture, 4242, 1);
        let grown = ProcFixture::new(4242, 30, 5, 4_096, 3);
        let second = sample_at(&grown, 4242, 2);
        let series =
            build_process_resource_series(CampaignProcessRole::Daemon, 4242, vec![first, second])
                .expect("publishable series");
        assert_eq!(series.claim_level, "hypothesis");
        assert_eq!(series.samples.len(), 2);
    }

    #[test]
    fn decreasing_cumulative_counters_are_treated_as_pid_reuse() {
        let high = ProcFixture::new(4242, 50, 9, 8_192, 3);
        let low = ProcFixture::new(4242, 1, 1, 1_024, 3);
        let error = build_process_resource_series(
            CampaignProcessRole::Daemon,
            4242,
            vec![sample_at(&high, 4242, 1), sample_at(&low, 4242, 2)],
        )
        .unwrap_err();
        assert_eq!(error, ResourceSamplerError::ReusedProcessIdentity);
    }

    #[test]
    fn empty_unordered_or_cross_identity_series_fail_closed() {
        assert_eq!(
            build_process_resource_series(CampaignProcessRole::Daemon, 1, Vec::new()).unwrap_err(),
            ResourceSamplerError::UnorderedSeries
        );
        let fixture = ProcFixture::new(4242, 1, 1, 1, 1);
        let sample = sample_at(&fixture, 4242, 1);
        assert_eq!(
            build_process_resource_series(CampaignProcessRole::Daemon, 4242, vec![sample, sample])
                .unwrap_err(),
            ResourceSamplerError::UnorderedSeries
        );
        assert_eq!(
            build_process_resource_series(CampaignProcessRole::PiSidecar, 4242, vec![sample])
                .unwrap_err(),
            ResourceSamplerError::MixedProcessIdentity
        );
    }

    #[test]
    fn self_promoted_series_is_not_publishable() {
        let fixture = ProcFixture::new(4242, 1, 1, 1, 1);
        let mut series = build_process_resource_series(
            CampaignProcessRole::Daemon,
            4242,
            vec![sample_at(&fixture, 4242, 1)],
        )
        .expect("publishable series");
        series.claim_level = "tested-local";
        assert_eq!(
            validate_process_resource_series(&series).unwrap_err(),
            ResourceSamplerError::ClaimShapedSeries
        );
    }
}
