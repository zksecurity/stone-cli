use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;

use sha2::Digest;

const ARTIFACTS: &str = "artifacts";
const RESOURCES: &str = "resources";

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

fn target_dir() -> Result<PathBuf, anyhow::Error> {
    match std::env::var("CARGO_TARGET_DIR") {
        Ok(dir) => Ok(dir.into()),
        Err(_) => {
            let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")?.into();
            Ok(manifest_dir.join("target"))
        }
    }
}

fn out_dir() -> Result<PathBuf, anyhow::Error> {
    Ok(std::env::var("OUT_DIR")?.into())
}

// directory for cached artifacts
fn artifact_store_dir() -> Result<PathBuf, anyhow::Error> {
    Ok(target_dir()?.join(ARTIFACTS))
}

fn resource_dir() -> Result<PathBuf, anyhow::Error> {
    Ok(target_dir()?.join(RESOURCES))
}

fn path_resource_tar() -> Result<PathBuf, anyhow::Error> {
    Ok(resource_dir()?.join("resources.tar.gz"))
}

fn path_resource_hash() -> Result<PathBuf, anyhow::Error> {
    Ok(resource_dir()?.join("resources-hash.txt"))
}

fn path_resources_rs() -> Result<PathBuf, anyhow::Error> {
    Ok(out_dir()?.join("resources.rs"))
}

fn ensure<T: AsRef<Path>>(path: T) -> Result<T, anyhow::Error> {
    match std::fs::create_dir_all(path.as_ref()) {
        Ok(_) => Ok(path),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(path),
        Err(e) => Err(e.into()),
    }
}

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
    fn path(&self) -> Result<PathBuf, anyhow::Error> {
        Ok(artifact_store_dir()?.join(self.id()))
    }
}

impl Artifact {
    // artifacts are hash-addressable
    fn id(&self) -> String {
        format!("sha256-{}", self.sha256_sum)
    }

    // open the artifact file
    fn file(&self) -> Result<std::fs::File, anyhow::Error> {
        std::fs::File::open(self.path()?).map_err(Into::into)
    }

    // check if the artifact exists
    fn exists(&self) -> bool {
        self.path().map(|p| p.exists()).unwrap_or(false)
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
    fn fetch(&mut self, artifacts: &[Artifact]) -> Result<(), anyhow::Error> {
        // create the artifact directory if it doesn't exist
        ensure(artifact_store_dir()?)?;

        // download every required artifact to the artifact store
        let client = reqwest::blocking::Client::new();
        for artifact in artifacts.iter() {
            // check if already exists
            if !artifact.exists() {
                // download the file
                println!("cargo:info=downloading artifact: {}", artifact.name);
                let resp = client.get(artifact.url).send()?;

                // check sha256 in-memory
                let bytes = resp.bytes()?;
                let bytes: &[u8] = bytes.as_ref();
                let hash = sha2::Sha256::digest(bytes);
                assert_eq!(
                    format!("{:x}", hash),
                    artifact.sha256_sum,
                    "Invalid sha256 sum of artifact {:?}",
                    artifact
                );

                // cache artifact to disk
                let mut file = std::fs::File::create(artifact.path()?)?;
                file.write_all(bytes).expect("Failed to write to file");
            }

            // add to the artifact store
            match self
                .artifacts
                .insert(artifact.name.to_owned(), artifact.clone())
            {
                Some(_) => anyhow::bail!("Duplicate artifact name: {}", artifact.name),
                None => (),
            }
        }

        Ok(())
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
    let tar_path = art.path()?;
    let tar_gz = std::fs::File::open(&tar_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(&tmp)?;
    Ok(tmp)
}

fn build_resource_tar(arts: &ArtifactStore) -> Result<(), anyhow::Error> {
    const DIR_EXEC: &str = "executables";
    const DIR_CORELIB: &str = "corelib";

    // create the resource directory
    ensure(resource_dir()?)?;

    // create the tarball
    let tar_gz = std::fs::File::create(path_resource_tar()?)?;
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

    // hash the tarball
    std::fs::write(
        path_resource_hash()?,
        format!("{}", hash(std::fs::read(path_resource_tar()?)?)),
    )?;
    Ok(())
}

fn generate_resources_rs() -> Result<(), anyhow::Error> {
    let tar_resource_path = path_resource_tar().expect("Failed to get tarball path");

    // read the hash of the tarball
    let hash = std::fs::read_to_string(path_resource_hash()?)?;
    let hash: u64 = hash.trim().parse()?;

    // create the resources.rs file with the tarball as a byte arrays
    let mut fl = std::fs::File::create(path_resources_rs()?)?;

    // read the tar and hash it to get the resource id
    writeln!(fl, "// Identifies the resources tarball")?;
    writeln!(fl, "pub const RESOURCE_ID: u64 = 0x{:x};", hash)?;
    writeln!(fl)?;

    // write the tarball as a byte array
    writeln!(fl, "// The resources tarball (bytes)")?;
    writeln!(
        fl,
        "pub const RESOURCE_TAR: &[u8] = include_bytes!(\"{}\");",
        tar_resource_path.display()
    )?;
    writeln!(fl)?;
    Ok(())
}

fn main() {
    // look up the stone-prover distribution for the current OS and architecture
    let os = OS.try_into().unwrap();
    let arch = ARCH.try_into().unwrap();
    let mut arts = ArtifactStore::default();
    if let Some(dist) = DISTS.get(&(os, arch)) {
        arts.fetch(dist).expect("Failed to fetch artifacts");
    } else {
        panic!("Unsupported OS or architecture {}/{}", OS, ARCH);
    }

    // create the resource tarball which has the whole directory structure
    build_resource_tar(&arts).expect("Failed to build resource tarball");

    // generate the resources.rs file
    generate_resources_rs().expect("Failed to generate resources.rs");

    // tell cargo to rerun the build script if the resources change
    println!("cargo:rerun-if-changed=build.rs");
}
