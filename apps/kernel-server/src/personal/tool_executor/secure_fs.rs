//! Handle-relative, no-follow filesystem operations for native workspace tools.
//!
//! Path validation alone is only an observation: another process can replace a
//! checked component before the later pathname open.  This module turns the
//! daemon-approved workspace root into a directory capability, descends one
//! component at a time without following links, and verifies the object type
//! from the opened handle before any read, write, or enumeration.

#[cfg(unix)]
use cap_fs_ext::OpenOptionsSyncExt;
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt};
use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use std::{
    ffi::{OsStr, OsString},
    fs::{File, Metadata},
    io,
    path::{Component, Path, PathBuf},
};

/// The Windows file-attribute bit set for every reparse point, including
/// junctions and mount points which `FileType::is_symlink` does not cover.
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

pub(crate) enum SecureEntry {
    Absent,
    Rejected,
    File(File),
    Directory(Dir),
}

pub(crate) struct AnchoredWorkspace {
    root: Dir,
}

impl AnchoredWorkspace {
    pub(crate) fn open(root_path: &Path) -> io::Result<Self> {
        let root = open_absolute_directory_nofollow(root_path)?;
        Ok(Self { root })
    }

    pub(crate) fn root_dir(&self) -> io::Result<Dir> {
        self.root.try_clone()
    }

    pub(crate) fn open_directory(&self, relative_path: &Path) -> io::Result<Dir> {
        open_directory_chain(self.root.try_clone()?, relative_path)
    }

    pub(crate) fn open_parent(&self, relative_path: &Path) -> io::Result<(Dir, OsString)> {
        let file_name = relative_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no file name"))?
            .to_os_string();
        let parent_path = relative_path.parent().unwrap_or_else(|| Path::new(""));
        let parent = self.open_directory(parent_path)?;
        Ok((parent, file_name))
    }

    pub(crate) fn open_entry(&self, relative_path: &Path) -> io::Result<SecureEntry> {
        if relative_path.as_os_str().is_empty() {
            return Ok(SecureEntry::Directory(self.root.try_clone()?));
        }
        let (parent, file_name) = self.open_parent(relative_path)?;
        open_entry_at(&parent, &file_name)
    }
}

pub(crate) fn open_entry_at(parent: &Dir, name: &OsStr) -> io::Result<SecureEntry> {
    validate_single_component(name)?;
    match parent.symlink_metadata(name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(SecureEntry::Absent);
        }
        Err(error) => return Err(error),
        Ok(metadata) => {
            if metadata.is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
                return Ok(SecureEntry::Rejected);
            }
        }
    }

    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.maybe_dir(true);
    #[cfg(unix)]
    options.nonblock(true);
    let opened = match parent.open_with(name, &options) {
        Ok(opened) => opened,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(SecureEntry::Absent);
        }
        Err(error) => {
            return match parent.symlink_metadata(name) {
                Ok(metadata)
                    if metadata.is_symlink() || (!metadata.is_file() && !metadata.is_dir()) =>
                {
                    Ok(SecureEntry::Rejected)
                }
                _ => Err(error),
            };
        }
    };
    let opened = opened.into_std();
    let metadata = opened.metadata()?;
    if is_windows_reparse(&metadata) {
        return Ok(SecureEntry::Rejected);
    }
    if metadata.is_file() {
        return Ok(SecureEntry::File(opened));
    }
    if metadata.is_dir() {
        return Ok(SecureEntry::Directory(Dir::from_std_file(opened)));
    }
    Ok(SecureEntry::Rejected)
}

pub(crate) fn create_new_regular_file(parent: &Dir, name: &OsStr) -> io::Result<File> {
    validate_single_component(name)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let opened = parent.open_with(name, &options)?.into_std();
    let metadata = opened.metadata()?;
    if !metadata.is_file() || is_windows_reparse(&metadata) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "new file is not a regular non-reparse file",
        ));
    }
    Ok(opened)
}

pub(crate) fn open_or_create_regular_file(parent: &Dir, name: &OsStr) -> io::Result<File> {
    validate_single_component(name)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    options.follow(FollowSymlinks::No);
    let opened = parent.open_with(name, &options)?.into_std();
    let metadata = opened.metadata()?;
    if !metadata.is_file() || is_windows_reparse(&metadata) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "opened entry is not a regular non-reparse file",
        ));
    }
    Ok(opened)
}

pub(crate) fn remove_regular_file(parent: &Dir, name: &OsStr) -> io::Result<bool> {
    validate_single_component(name)?;
    let metadata = match parent.symlink_metadata(name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
        Ok(metadata) => metadata,
    };
    if !metadata.is_file() || metadata.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "refusing to remove a non-regular staging entry",
        ));
    }
    parent.remove_file(name)?;
    Ok(true)
}

pub(crate) fn directory_identity(directory: &Dir) -> io::Result<FileIdentity> {
    let metadata = directory.try_clone()?.into_std_file().metadata()?;
    file_identity(&metadata)
}

pub(crate) fn sync_directory(directory: &Dir) -> io::Result<()> {
    #[cfg(unix)]
    {
        return directory.try_clone()?.into_std_file().sync_all();
    }
    #[cfg(not(unix))]
    {
        let _ = directory;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileIdentity {
    first: u64,
    second: u64,
}

#[cfg(unix)]
fn file_identity(metadata: &Metadata) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    Ok(FileIdentity {
        first: metadata.dev(),
        second: metadata.ino(),
    })
}

#[cfg(windows)]
fn file_identity(metadata: &Metadata) -> io::Result<FileIdentity> {
    use std::os::windows::fs::MetadataExt;
    let volume = metadata.volume_serial_number().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "workspace handle has no volume identity",
        )
    })?;
    let index = metadata.file_index().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "workspace handle has no file identity",
        )
    })?;
    Ok(FileIdentity {
        first: u64::from(volume),
        second: index,
    })
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_metadata: &Metadata) -> io::Result<FileIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "workspace handle identity is unsupported on this platform",
    ))
}

#[cfg(windows)]
fn is_windows_reparse(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse(_metadata: &Metadata) -> bool {
    false
}

fn open_directory_chain(mut current: Dir, relative_path: &Path) -> io::Result<Dir> {
    for component in relative_path.components() {
        let Component::Normal(component) = component else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "workspace path contains a non-normal component",
            ));
        };
        current = match open_entry_at(&current, component)? {
            SecureEntry::Directory(directory) => directory,
            SecureEntry::Absent => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "workspace directory component disappeared",
                ));
            }
            SecureEntry::File(_) | SecureEntry::Rejected => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "workspace directory component is a file, link, or reparse point",
                ));
            }
        };
    }
    Ok(current)
}

fn open_absolute_directory_nofollow(path: &Path) -> io::Result<Dir> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut ambient_root = PathBuf::new();
    let mut descendants = Vec::new();
    for component in absolute_path.components() {
        match component {
            Component::Prefix(prefix) => ambient_root.push(prefix.as_os_str()),
            Component::RootDir => ambient_root.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(name) => descendants.push(name.to_os_string()),
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "workspace root contains a parent traversal component",
                ));
            }
        }
    }
    if ambient_root.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace root has no absolute filesystem anchor",
        ));
    }
    let mut current = Dir::open_ambient_dir(ambient_root, ambient_authority())?;
    for descendant in descendants {
        current = match open_entry_at(&current, &descendant)? {
            SecureEntry::Directory(directory) => directory,
            SecureEntry::Absent => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "workspace root component disappeared",
                ));
            }
            SecureEntry::File(_) | SecureEntry::Rejected => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "workspace root component is a file, link, or reparse point",
                ));
            }
        };
    }
    Ok(current)
}

fn validate_single_component(name: &OsStr) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "filesystem operation requires one normal path component",
        ));
    }
    Ok(())
}
