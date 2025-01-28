use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;
use thiserror::Error;

use sha2::Digest;

// cached artifact directory
const ARTIFACT_DIR: &str = "target/artifacts";

// these are just arbitrary labels for each resource
// they map to different artifacts for different OS and architectures
const RES_STONE_V5_PROVER: &str = "bin:v5-prover";
const RES_STONE_V6_PROVER: &str = "bin:v6-prover";
const RES_STONE_V5_VERIFIER: &str = "bin:v5-verifier";
const RES_STONE_V6_VERIFIER: &str = "bin:v6-verifier";
const RES_CAIRO_1RUN: &str = "tar-gz:cairo1-run";
const RES_CORELIB: &str = "tar-gz:corelib";

// binary names
const BIN_STONE_V5_PROVER: &str = "cpu_air_prover_v5";
const BIN_STONE_V6_PROVER: &str = "cpu_air_prover_v6";
const BIN_STONE_V5_VERIFIER: &str = "cpu_air_verifier_v5";
const BIN_STONE_V6_VERIFIER: &str = "cpu_air_verifier_v6";
const BIN_CAIRO_1RUN: &str = "cairo1-run";

// excutables to add to the resources
const EXECUTABLES: [(&str, &str); 4] = [
    (RES_STONE_V5_PROVER, BIN_STONE_V5_PROVER),
    (RES_STONE_V5_VERIFIER, BIN_STONE_V5_VERIFIER),
    (RES_STONE_V6_PROVER, BIN_STONE_V6_PROVER),
    (RES_STONE_V6_VERIFIER, BIN_STONE_V6_VERIFIER),
];

// list of artifacts for different OS and architectures
static DISTS: Lazy<HashMap<(Os, Arch), Vec<Artifact>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        (Os::Linux, Arch::Amd64),
        vec![
            // TODO: deprecate this
            Artifact {
                name: RES_CAIRO_1RUN,
                url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0/cairo1-run-v2.0.0-rc0-x86_64.tar.gz",
                sha256_sum: "bbdcaad15bf44e7b4b8e2eadb7b8287787bbf738acb60ad0f87f991cadfa56a4",
            },
            Artifact {
                name: RES_STONE_V5_PROVER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_prover-x86_64",
                sha256_sum: "d5345e3e72a6180dabcec79ef35cefc735ea72864742e1cc117869da7d122ee5",
            },
            Artifact {
                name: RES_STONE_V5_VERIFIER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_verifier-x86_64",
                sha256_sum: "8ed3cad6cf3fb10f5a600af861c28b8f427244b0c2de920f1c18ea78371a66a9",
            },
            Artifact {
                name: RES_STONE_V6_PROVER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_prover-x86_64",
                sha256_sum: "ec33129a15b888b7946f17fe46ca888bfed2f4d86ac4e3fc7fae787f8162ca9e",
            },
            Artifact {
                name: RES_STONE_V6_VERIFIER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_verifier-x86_64",
                sha256_sum: "f83d66f5f9cd60c070fee02524d4ccb86b1c37865d75c022fbd54c349d7d972b",
            },
            Artifact {
                name: RES_CORELIB,
                url: "https://github.com/starkware-libs/cairo/releases/download/v2.9.0-dev.0/release-x86_64-unknown-linux-musl.tar.gz",
                sha256_sum: "d52c7acd29bd83762aa60974bc3b28ad260eb2d822952c186ec6f12994d3c824"
            }
        ],
    );
    m.insert(
        (Os::MacOS, Arch::Aarch64),
        vec![
            // TODO: deprecate this
            Artifact {
                name: RES_CAIRO_1RUN,
                url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0/cairo1-run-v2.0.0-rc0-aarch64.tar.gz",
                sha256_sum: "ff682b3e91c9447e5719b0989abbeeec56f4b68a0ce3de843fb8e52b19f93cb0",
            },
            Artifact {
                name: RES_STONE_V5_PROVER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_prover-arm64",
                sha256_sum: "d91e8328b7a228445dda0b9d1acb21a86ab894727737e2d70a0210179b90f00e",
            },
            Artifact {
                name: RES_STONE_V5_VERIFIER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_verifier-arm64",
                sha256_sum: "fc4090e3395e101f3481efc247ad590e5db7704c31321480522904d68ba5d009",
            },
            Artifact {
                name: RES_STONE_V6_PROVER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_prover-arm64",
                sha256_sum: "9d56eaa56eda5caa6853761f93d363dc3e9e9af27cf142cd0178dbcd4f61d405",
            },
            Artifact {
                name: RES_STONE_V6_VERIFIER,
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_verifier-arm64",
                sha256_sum: "bfd92c9f8c6be41a0486c936b0f12df153ee2743edbf782e21f15fa56e3bdb70",
            },
            Artifact {
                name: RES_CORELIB,
                url: "https://github.com/starkware-libs/cairo/releases/download/v2.9.0-dev.0/release-x86_64-unknown-linux-musl.tar.gz",
                sha256_sum: "d52c7acd29bd83762aa60974bc3b28ad260eb2d822952c186ec6f12994d3c824"
            }
        ],
    );
    m
});

#[derive(Debug, Error)]
enum ConversionError {
    #[error("Unsupported architecture: {0}")]
    UnsupportedArchitecture(String),
    #[error("Unsupported operating system: {0}")]
    UnsupportedOperatingSystem(String),
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Os {
    Linux,
    MacOS,
}

impl TryInto<Os> for &str {
    type Error = ConversionError;

    fn try_into(self) -> Result<Os, Self::Error> {
        match self {
            "linux" => Ok(Os::Linux),
            "macos" => Ok(Os::MacOS),
            _ => Err(ConversionError::UnsupportedOperatingSystem(
                self.to_string(),
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Arch {
    Aarch64,
    Amd64,
}

impl TryInto<Arch> for &str {
    type Error = ConversionError;

    fn try_into(self) -> Result<Arch, Self::Error> {
        match self {
            "aarch64" => Ok(Arch::Aarch64),
            "x86_64" => Ok(Arch::Amd64),
            _ => Err(ConversionError::UnsupportedArchitecture(self.to_string())),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct Artifact {
    url: &'static str,
    name: &'static str,
    sha256_sum: &'static str,
}

impl Artifact {
    fn path(&self) -> std::path::PathBuf {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(ARTIFACT_DIR)
            .join(&self.id())
    }
}

impl Artifact {
    // artifacts are hash-addressable
    fn id(&self) -> String {
        format!("sha256-{}", self.sha256_sum)
    }

    //
    fn file(&self) -> Result<std::fs::File, std::io::Error> {
        std::fs::File::open(self.path())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ArtifactStore {
    artifacts: HashMap<String, Artifact>,
}

impl Hash for ArtifactStore {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut elems: Vec<_> = self.artifacts.iter().collect();
        elems.sort();
        elems.hash(state);
    }
}

impl ArtifactStore {
    /// Add the given artifacts to the artifact store.
    fn fetch(&mut self, artifacts: &[Artifact]) {
        // create the artifact directory if it doesn't exist
        let art_dir = Path::new(ARTIFACT_DIR);
        if !art_dir.exists() {
            std::fs::create_dir_all(art_dir).expect("Failed to create target directory");
        }

        // download every required artifact to the artifact store
        let client = reqwest::blocking::Client::new();
        for artifact in artifacts.iter() {
            // check if already exists
            if !art_dir.join(&artifact.id()).exists() {
                // download the file
                let resp = client
                    .get(artifact.url)
                    .send()
                    .expect("Failed to download file");

                // check sha256 in-memory
                let bytes = resp.bytes().unwrap();
                let bytes: &[u8] = bytes.as_ref();
                let hash = sha2::Sha256::digest(bytes);
                assert_eq!(
                    format!("{:x}", hash),
                    artifact.sha256_sum,
                    "Invalid sha256 sum of artifact {:?}",
                    artifact
                );

                // cache artifact to disk
                let mut file =
                    std::fs::File::create(&artifact.path()).expect("Failed to create file");
                std::io::copy(&mut std::io::Cursor::new(bytes), &mut file)
                    .expect("Failed to write to file");
            }

            // add to the artifact store
            assert!(
                self.artifacts
                    .insert(artifact.name.to_owned(), artifact.clone())
                    .is_none(),
                "Duplicate artifact name {}",
                artifact.name
            );
        }
    }

    /// Find the artifact with the given name.
    fn find(&self, name: &str) -> Result<&Artifact, anyhow::Error> {
        match self.artifacts.get(name) {
            Some(artifact) => Ok(artifact),
            None => Err(anyhow::anyhow!("Failed to find artifact {}", name)),
        }
    }
}

fn hash<T: Hash>(t: T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn archive_add_exe(
    archive: &mut tar::Builder<flate2::write::GzEncoder<std::fs::File>>,
    mut file: std::fs::File,
    name: &str,
) -> Result<(), std::io::Error> {
    let mut perm = file.metadata()?.permissions();
    perm.set_mode(0o755);
    file.set_permissions(perm)?;
    archive.append_file(name, &mut file)?;
    Ok(())
}

fn deflate_artifact(art: &Artifact) -> Result<TempDir, anyhow::Error> {
    let tmp = TempDir::new()?;
    let tar_path = art.path();
    let tar_gz = std::fs::File::open(&tar_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(&tmp)?;
    Ok(tmp)
}

fn build_resource_tar(arts: &ArtifactStore) -> Result<(), anyhow::Error> {
    const DIR_EXEC: &str = "executables";
    const DIR_CORELIB: &str = "corelib";

    // check cache: it is expensive to build the tarball
    // and so we want to avoid doing it every time
    // we change the stone-cli
    //
    // note: because Cargo takes a lock on the project,
    // the section below need not be thread-safe
    let cache_id: String = format!(
        "{:x}",
        hash((
            arts,                       // the artifacts
            std::fs::read("build.rs")?  // the build script
        ))
    );
    let txt_version_path = std::env::current_dir()?
        .join("target")
        .join("resources-version.txt");
    let tar_resource_path = std::env::current_dir()?
        .join("target")
        .join("resources.tar.gz");

    // check if the version is the cache_id
    if txt_version_path.exists() {
        if cache_id == std::fs::read_to_string(&txt_version_path)?.trim() {
            return Ok(());
        }
    }

    // create the tarball
    let tar_gz = std::fs::File::create(&tar_resource_path)?;
    let tar = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut archive = tar::Builder::new(tar);

    // decompress the corelib tarball and add "cario/corelib" as "corelib" to the archive
    archive.append_dir_all(
        DIR_CORELIB,
        &deflate_artifact(arts.find(RES_CORELIB)?)?
            .path()
            .join("cairo/corelib"),
    )?;

    // decompress cairo1-run tarball and add "cairo1-run" to the "executables" directory
    {
        // find the inner dir
        let tmp = deflate_artifact(arts.find(RES_CAIRO_1RUN)?)?;
        let entries: Vec<_> = std::fs::read_dir(tmp.path())?.map(|e| e.unwrap()).collect();
        assert_eq!(
            entries.len(),
            1,
            "Invalid number of entries in cairo1-run archive"
        );

        // copy the cairo1-run executable to the archive
        let src = entries[0].path().join("cairo1-run-v2.0.0-rc0");
        let dst = Path::new(DIR_EXEC).join(BIN_CAIRO_1RUN);
        assert!(src.exists(), "{}", src.display());
        archive_add_exe(
            &mut archive,
            std::fs::File::open(src)?,
            dst.to_str().unwrap(),
        )?;
    }

    // copy all the executables
    for (res, bin) in EXECUTABLES {
        let art = arts.find(res)?;
        let dst = Path::new(DIR_EXEC).join(bin);
        archive_add_exe(&mut archive, art.file()?, dst.to_str().unwrap())?;
    }

    // finish the archive
    archive.finish()?;

    // create the resources.rs file with the tarball as a byte arrays
    {
        let mut fl = std::fs::File::create("src/resources.rs")?;
        writeln!(fl, "//! This file is generated by build.rs")?;
        writeln!(fl)?;

        // read the tar and hash it to get the resource id
        writeln!(fl, "// Identifies the resources tarball")?;
        writeln!(fl, "pub const RESOURCE_ID: u64 = 0x{:x};", {
            hash(std::fs::read(&tar_resource_path)?)
        })?;
        writeln!(fl)?;

        // write the tarball as a byte array
        writeln!(fl, "// The resources tarball (bytes)")?;
        writeln!(
            fl,
            "pub const RESOURCE_TAR: &[u8] = include_bytes!(\"{}\");",
            tar_resource_path.display()
        )?;
        writeln!(fl)?;
    }

    // mark the cache version
    std::fs::write(&txt_version_path, cache_id)?;
    Ok(())
}

fn main() {
    // look up the stone-prover distribution for the current OS and architecture
    let os = OS.try_into().unwrap();
    let arch = ARCH.try_into().unwrap();
    let mut arts = ArtifactStore::default();
    if let Some(dist) = DISTS.get(&(os, arch)) {
        arts.fetch(dist);
    } else {
        panic!("Unsupported OS or architecture {}/{}", OS, ARCH);
    }

    // create the resource tarball which has the whole directory structure
    build_resource_tar(&arts).expect("Failed to build resource tarball");
}
