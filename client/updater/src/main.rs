mod local_fs_impl;
use std::{
    env::temp_dir,
    fs::{self, File, OpenOptions},
    process::exit,
    time::Duration,
};

use clap::{Parser, Subcommand};
use firststage::{
    core::{validate_and_perform_update, validate_update},
    structs::Semver,
};
use reqwest::{Certificate, Url};

use crate::local_fs_impl::{
    FSUpdateFileProvider, FSUpdateEffector, FSKeyProvider, TRIGGER_UPDATE_FILE,
};

/// Package an installer into an xdu file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Keys directory
    #[arg(long)]
    key_directory: String,

    /// Current version
    #[arg(long)]
    current_version: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check if the update is valid
    VerifyFile {
        /// xdu file to check
        file: String,
    },
    /// Check if the update is valid
    ExtractFile {
        /// xdu file to extract
        file: String,

        // destination (output file)
        destination: String,
    },

    /// Check for updates from the server
    Check {
        /// Output installer, if validation is OK
        #[arg(long)]
        installer_to_write: String,

        /// Installation serial
        #[arg(long)]
        serial: String,

        /// Certificate directory - will enable cert pinning.
        #[arg(long)]
        cert_dir: Option<String>,

        /// Timeout (in millis). Defaults to 2000
        #[arg(long)]
        timeout: Option<u64>,

        /// Base Server URL
        server: String,
    },
}

fn main() {
    /*let args = Cli {
        command: Commands::VerifyFile { file: String::from("/ram/output") }, current_version: String::from("0.0.0"), key_directory: String::from("/ram/")
    }; // Cli::parse(); */
    let args = Cli::parse();
    let key_provider = FSKeyProvider::new(&args.key_directory).unwrap();
    let current_ver = Semver::parse(&args.current_version).unwrap();
    match args.command {
        Commands::VerifyFile { file } => {
            let file = File::open(file).unwrap();
            let mut source = FSUpdateFileProvider::new(file);
            let dummy_effector = FSUpdateEffector::new_validation_only(current_ver);
            if let Err(x) = validate_update(&key_provider, &mut source, &dummy_effector) {
                println!("Error! {x:?}");
            } else {
                println!("Update file valid for the current version.");
            }
        }
        Commands::ExtractFile { file, destination } => {
            let file = File::open(file).unwrap();
            let mut source = FSUpdateFileProvider::new(file);
            let effector = FSUpdateEffector::new(current_ver, &destination);
            if let Err(x) = validate_and_perform_update(&key_provider, &mut source, &effector) {
                println!("Error! {x:?}");
            } else {
                println!("Update file extracted.");
            }
        }
        Commands::Check {
            installer_to_write,
            serial,
            cert_dir,
            server: base_server_url,
            timeout,
        } => {
            let mut client = reqwest::blocking::ClientBuilder::new()
                .timeout(Some(Duration::from_millis(timeout.unwrap_or(2000))));
            if let Some(cert_dir) = cert_dir {
                let mut certs = vec![];
                for file in fs::read_dir(cert_dir).unwrap() {
                    let entry = file.unwrap();
                    match Certificate::from_der(&fs::read(entry.path()).unwrap()) {
                        Err(error) => println!(
                            "Error parsing certificate {}: {error:?}",
                            entry.path().display()
                        ),
                        Ok(cert) => {
                            certs.push(cert);
                        }
                    }
                }
                client = client.tls_certs_only(certs);
            }

            // DISABLE IN PRODUCTION ENVIRONMENT!!! :
            client = client.tls_danger_accept_invalid_certs(true);

            let client = client.build().unwrap();
            let base_url = Url::parse(&base_server_url).unwrap();
            if base_url.scheme() != "https" {
                panic!("The updater must use HTTPS to communicate with the server!");
            }

            let mut ver_query_url = base_url.clone();
            ver_query_url.set_path("/get_newest");
            ver_query_url
                .query_pairs_mut()
                .append_pair("serial", &serial);

            let response = client.get(ver_query_url).send().unwrap().text().unwrap();
            // Should respond with:
            // <semver> <file to download>
            let parts: Vec<_> = response.split(' ').collect();
            let semver = if parts.len() == 2 {
                match Semver::parse(parts[0]) {
                    Ok(e) => e,
                    Err(e) => {
                        panic!("Server sent invalid semver: {e}");
                    }
                }
            } else {
                panic!("Invalid response from server");
            };
            if semver <= current_ver {
                println!("Up to date.");
                return;
            }
            // Not up to date. Fetch the file.
            let mut file_fetch_url = base_url.clone();
            file_fetch_url.path_segments_mut().unwrap().push("files");
            file_fetch_url.path_segments_mut().unwrap().push(parts[1]);
            let mut update_file_response = client.get(file_fetch_url).send().unwrap();
            let temp_file_path = temp_dir().join(format!("temporary-{serial}.xdu"));
            let mut temporary_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .read(true)
                .open(&temp_file_path)
                .unwrap();
            update_file_response.copy_to(&mut temporary_file).unwrap();
            let effector = FSUpdateEffector::new(current_ver, &installer_to_write);
            let mut source = FSUpdateFileProvider::new(temporary_file);
            validate_and_perform_update(&key_provider, &mut source, &effector).unwrap();

            // fs::remove_file(&temp_file_path).unwrap();
            exit(TRIGGER_UPDATE_FILE);
        }
    }
}
