use std::{env, process};

use ledger_manager::{
    genuine_check, install_app, install_bitcoin_app,
    ledger_transport_hidapi::{hidapi::HidApi, TransportNativeHID},
    list_installed_apps, open_app, open_bitcoin_app, update_app, update_bitcoin_app, DeviceInfo,
    InstallErr, LedgerApp, UpdateErr,
};

// Print on stderr and exit with 1.
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        process::exit(1);
    }};
}

#[derive(Debug, Clone, Copy)]
enum Command {
    GetInfo,
    GenuineCheck,
    InstallMainApp,
    UpdateMainApp,
    OpenMainApp,
    InstallTestApp,
    UpdateTestApp,
    OpenTestApp,
    InstallSolana,
    UpdateSolana,
    OpenSolana,
    UpdateFirmware,
}

impl Command {
    /// Read command from environment variables.
    pub fn get() -> Option<Self> {
        let is_testnet = env::var("LEDGER_TESTNET").is_ok();
        let is_solana = env::var("LEDGER_SOLANA").is_ok();
        let cmd_str = env::var("LEDGER_COMMAND").ok()?;

        if cmd_str == "getinfo" {
            Some(Self::GetInfo)
        } else if cmd_str == "genuinecheck" {
            Some(Self::GenuineCheck)
        } else if cmd_str == "installapp" {
            if is_solana {
                Some(Self::InstallSolana)
            } else if is_testnet {
                Some(Self::InstallTestApp)
            } else {
                Some(Self::InstallMainApp)
            }
        } else if cmd_str == "updateapp" {
            if is_solana {
                Some(Self::UpdateSolana)
            } else if is_testnet {
                Some(Self::UpdateTestApp)
            } else {
                Some(Self::UpdateMainApp)
            }
        } else if cmd_str == "openapp" {
            if is_solana {
                Some(Self::OpenSolana)
            } else if is_testnet {
                Some(Self::OpenTestApp)
            } else {
                Some(Self::OpenMainApp)
            }
        } else if cmd_str == "updatefirm" {
            Some(Self::UpdateFirmware)
        } else {
            None
        }
    }
}

fn ledger_api() -> TransportNativeHID {
    let hid_api = match HidApi::new() {
        Ok(a) => a,
        Err(e) => error!("Error initializing HDI api: {}.", e),
    };
    match TransportNativeHID::new(&hid_api) {
        Ok(a) => a,
        Err(e) => error!("Error connecting to Ledger device: {}.", e),
    }
}

fn device_info(ledger_api: &TransportNativeHID) -> DeviceInfo {
    match DeviceInfo::new(ledger_api) {
        Ok(i) => i,
        Err(e) => error!("Error fetching device info: {}", e),
    }
}

fn print_ledger_info(ledger_api: &TransportNativeHID) {
    let device_info = device_info(ledger_api);
    println!("Information about the device: {:#?}", device_info);

    println!("Querying installed applications from your Ledger. You might have to confirm on your device.");
    let apps = match list_installed_apps(ledger_api) {
        Ok(a) => a,
        Err(e) => error!("Error listing installed applications: {}.", e),
    };
    println!("Installed applications:");
    for app in apps {
        println!("  - {:?}", app);
    }
}

fn perform_genuine_check(ledger_api: &TransportNativeHID) {
    println!("Querying Ledger's remote HSM to perform the genuine check. You might have to confirm the operation on your device.");
    if let Err(e) = genuine_check(ledger_api) {
        error!("Error when performing genuine check: {}", e);
    }
    println!("Success. Your Ledger is genuine.");
}

// Install the Bitcoin app on the device.
fn install_bitcoin(ledger_api: &TransportNativeHID, is_testnet: bool) {
    println!("You may have to allow on your device 1) listing installed apps 2) the Ledger manager to install the app.");
    match install_bitcoin_app(ledger_api, is_testnet) {
        Ok(()) => println!("Successfully installed the app."),
        Err(InstallErr::AlreadyInstalled) => {
            error!("Bitcoin app already installed. Use the update command to update it.")
        }
        Err(InstallErr::AppNotFound) => error!("Could not get info about Bitcoin app."),
        Err(InstallErr::Any(e)) => error!("Error installing Bitcoin app: {}.", e),
    }
}

fn update_bitcoin(ledger_api: &TransportNativeHID, is_testnet: bool) {
    println!("You may have to allow on your device 1) listing installed apps 2) the Ledger manager to install the app.");
    match update_bitcoin_app(ledger_api, is_testnet) {
        Ok(()) => println!("Successfully updated the app."),
        Err(UpdateErr::NotInstalled) => {
            error!("Bitcoin app isn't installed. Use the install command instead.")
        }
        Err(UpdateErr::AppNotFound) => error!("Could not get info about Bitcoin app."),
        Err(UpdateErr::AlreadyLatest) => error!("Bitcoin app is already at the latest version."),
        Err(UpdateErr::Any(e)) => error!("Error installing Bitcoin app: {}.", e),
    }
}

fn open_bitcoin(ledger_api: &TransportNativeHID, is_testnet: bool) {
    if let Err(e) = open_bitcoin_app(ledger_api, is_testnet) {
        error!("Error opening Bitcoin app: {}", e);
    }
}

// Install the Solana app on the device.
fn install_solana(ledger_api: &TransportNativeHID) {
    println!("You may have to allow on your device 1) listing installed apps 2) the Ledger manager to install the app.");
    match install_app(ledger_api, LedgerApp::Solana) {
        Ok(()) => println!("Successfully installed the Solana app."),
        Err(InstallErr::AlreadyInstalled) => {
            error!("Solana app already installed. Use the update command to update it.")
        }
        Err(InstallErr::AppNotFound) => error!("Could not get info about Solana app."),
        Err(InstallErr::Any(e)) => error!("Error installing Solana app: {}.", e),
    }
}

fn update_solana(ledger_api: &TransportNativeHID) {
    println!("You may have to allow on your device 1) listing installed apps 2) the Ledger manager to install the app.");
    match update_app(ledger_api, LedgerApp::Solana) {
        Ok(()) => println!("Successfully updated the Solana app."),
        Err(UpdateErr::NotInstalled) => {
            error!("Solana app isn't installed. Use the install command instead.")
        }
        Err(UpdateErr::AppNotFound) => error!("Could not get info about Solana app."),
        Err(UpdateErr::AlreadyLatest) => error!("Solana app is already at the latest version."),
        Err(UpdateErr::Any(e)) => error!("Error updating Solana app: {}.", e),
    }
}

fn open_solana(ledger_api: &TransportNativeHID) {
    if let Err(e) = open_app(ledger_api, LedgerApp::Solana) {
        error!("Error opening Solana app: {}", e);
    }
}

fn main() {
    let command = if let Some(cmd) = Command::get() {
        cmd
    } else {
        error!("Invalid or no command specified. The command must be passed through the LEDGER_COMMAND env var. Set LEDGER_TESTNET to use the Bitcoin testnet app instead where applicable.");
    };

    let ledger_api = ledger_api();
    match command {
        Command::GetInfo => {
            print_ledger_info(&ledger_api);
        }
        Command::GenuineCheck => {
            perform_genuine_check(&ledger_api);
        }
        Command::InstallMainApp => {
            install_bitcoin(&ledger_api, false);
        }
        Command::InstallTestApp => {
            install_bitcoin(&ledger_api, true);
        }
        Command::OpenMainApp => {
            open_bitcoin(&ledger_api, false);
        }
        Command::OpenTestApp => {
            open_bitcoin(&ledger_api, true);
        }
        Command::UpdateMainApp => {
            update_bitcoin(&ledger_api, false);
        }
        Command::UpdateTestApp => {
            update_bitcoin(&ledger_api, true);
        }
        Command::InstallSolana => {
            install_solana(&ledger_api);
        }
        Command::UpdateSolana => {
            update_solana(&ledger_api);
        }
        Command::OpenSolana => {
            open_solana(&ledger_api);
        }
        Command::UpdateFirmware => {
            unimplemented!()
        }
    }
}
