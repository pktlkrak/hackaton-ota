use std::fs::{self, File};
use std::io::{Read, Seek, Write};

use anyhow::{Result, bail};
use clap::{Subcommand, Parser};
use firststage::structs::{ADDITIONAL_METADATA_OFFSET, AdditionalMetadata, SECOND_STAGE_OFFSET, Semver};
use getrandom::SysRng;
use ml_dsa::{KeyGen, MlDsa87, signature::rand_core::UnwrapErr};
use ml_dsa::signature::Keypair;
use sha2::{Digest, Sha512};
use ml_dsa::signature::Signer;

/// Package an installer into an xdu file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a key pair
    CreateKeypair {
        /// Id of the key
        #[arg(long)]
        id: u64,

        file_path: String,
    },

    /// Package an installer
    Package {
        /// Private key file to sign with
        #[arg(long)]
        private_key_file: String,

        /// Version
        #[arg(long)]
        version: String,

        /// Installer to package 
        installer_path: String,

        /// Output file
        output: String
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::CreateKeypair { id, file_path } => {
            let mut rng = UnwrapErr(SysRng);
            let kp = MlDsa87::key_gen(&mut rng);
            let key_seed: [u8; 32] = kp.to_seed().try_into().unwrap();
            let veri_key = kp.verifying_key().encode();
            
            let mut priv_key = File::create(format!("{file_path}.prv")).unwrap();
            priv_key.write_all(&id.to_le_bytes()).unwrap();
            priv_key.write_all(&key_seed).unwrap();

            std::fs::write(format!("{file_path}.key"), veri_key).unwrap();
            println!("Keys generated")
        },
        Commands::Package { private_key_file, version, installer_path, output } => {
            if let Err(x) = package_file(private_key_file, version, installer_path, output.clone()) {
                // Make sure the output doesn't exist if it crashes.
                let _ = std::fs::remove_file(output);
                println!("Error: {x:?}");
            }
        }
    }
}


fn package_file(private_key_file: String, version: String, installer_path: String, output: String) -> Result<()> {
    let private_key_contents = fs::read(private_key_file)?;
    if private_key_contents.len() != (32 + 8) {
        bail!("Incorrect private key length!");
    }
    let mut installer_image = File::open(installer_path)?;
    let mut output = File::create(output)?;
    let installer_length = installer_image.metadata()?.len();

    let id = u64::from_le_bytes((&private_key_contents[0..8]).try_into().unwrap());
    let kp = MlDsa87::from_seed((&private_key_contents[8..32 + 8]).try_into().unwrap());
    let signing_key = kp.signing_key();

    let semver = Semver::parse(&version).unwrap();
    let additional_header = AdditionalMetadata {
        length: SECOND_STAGE_OFFSET + installer_length,
        semver,
    };
    let mut serialized_additional_header = [0u8; 16];
    additional_header.serialize(&mut serialized_additional_header);

    output.write_all(&vec![0; ADDITIONAL_METADATA_OFFSET as usize]).unwrap();
    output.write_all(&serialized_additional_header)?;
    let mut hasher = Sha512::new();

    let mut buffer: [u8; 512] = [0; 512];
    let mut cursor = 0;
    hasher.update(&serialized_additional_header);
    while cursor < installer_length {
        let part_size = 512u64.min(installer_length - cursor) as usize;
        installer_image.read_exact(&mut buffer[0..part_size])?;
        hasher.update(&mut buffer[0..part_size]);
        output.write_all(&mut buffer[0..part_size])?;
        cursor += part_size as u64;
    }

    // Create the SHA and sign it.
    let shasum = hasher.finalize().to_vec();
    assert_eq!(shasum.len(), 64);
    output.seek(std::io::SeekFrom::Start(0))?;
    output.write_all("UPXD0001".as_bytes())?;
    output.write_all(&id.to_le_bytes())?;
    output.write_all(&shasum)?;

    let message: Vec<u8> = signing_key.sign(&shasum).encode().to_vec();
    assert_eq!(message.len(), 4627);
    output.write_all(&message)?;

    Ok(())
}
