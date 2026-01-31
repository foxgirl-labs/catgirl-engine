use clap::Parser;
use utils::args::Args;

// Constants
#[cfg(target_os = "android")]
const TAG: &str = "CatgirlEngine";

/// Process args for future use
pub(super) fn process_args() {
    // Store resources path in separate variable
    #[cfg(feature = "client")]
    client::game::store_resources_path(get_args().resources);

    // Uninstall desktop files
    #[cfg(all(feature = "client", target_os = "linux"))]
    if get_args().uninstall_desktop_files {
        trace!("Uninstalling desktop files...");
        let _ = client::uninstall_desktop_files();
    }

    // Install desktop files
    #[cfg(all(feature = "client", target_os = "linux"))]
    if get_args().install_desktop_files {
        trace!("Installing desktop files...");
        let _ = client::install_desktop_files();
    }

    #[cfg(not(target_family = "wasm"))]
    if get_args().print_environment_variables {
        trace!("Printing environment variables...");
        utils::environment::print_environment_vars();
    }

    #[cfg(feature = "client")]
    trace!("Resources Path: {:?}", client::game::get_resources_path());
}

/// Retrieve parsed out command line arguments
///
/// # Panics
///
/// This may panic if the args cannot be unwrapped
#[must_use]
pub(super) fn get_args() -> Args {
    if utils::args::get_args().is_some() {
        utils::args::get_args().unwrap()
    } else {
        Args::parse()
    }
}

/// Setup the logger for the current platform
#[cfg(feature = "logging-subscriber")]
pub(super) fn setup_logger() {
//  TODO: Figure out how to alias "verbose" to act as "trace"

    if cfg!(target_os = "android") {
//      Limited Filter: trace,android_activity=debug,winit=debug
//      Stronger Filter: trace,android_activity=off,winit=off

        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(tracing::log::LevelFilter::Trace)
                .with_tag(TAG)
                .with_filter(
                    android_logger::FilterBuilder::new()
                        .parse("main=trace,catgirl_engine=trace")
                        .build(),
                ),
        );
    } else if cfg!(target_family = "wasm") {
//      https://github.com/daboross/fern/issues/134
        #[cfg(target_family = "wasm")]
        if let Err(_error) = fern::Dispatch::new()
            .level(tracing::log::LevelFilter::Off)
            .level_for("main", tracing::log::LevelFilter::Trace)
            .level_for("catgirl_engine", tracing::log::LevelFilter::Trace)
            .level_for("catgirl_engine_client", tracing::log::LevelFilter::Trace)
            .level_for("catgirl_engine_server", tracing::log::LevelFilter::Trace)
            .level_for("catgirl_engine_utils", tracing::log::LevelFilter::Trace)
            .chain(fern::Output::call(console_log::log))
            .apply()
        {
            warn!("Failed to initialize console logger...")
        }
    } else {
//      windows, unix (includes Linux, BSD, OSX), or target_os="macos"
        let mut builder = pretty_env_logger::formatted_builder();
        if let Ok(s) = std::env::var("RUST_LOG") {
//          Set logger according to RUST_LOG environment variable
            builder.parse_filters(&s);
        } else {
//          Failed to find RUST_LOG environment variable
            builder
                .default_format()
                .filter(Some("main"), tracing::log::LevelFilter::Info)
                .filter(Some("catgirl_engine"), tracing::log::LevelFilter::Info)
                .filter(
                    Some("catgirl_engine_client"),
                    tracing::log::LevelFilter::Info,
                )
                .filter(
                    Some("catgirl_engine_server"),
                    tracing::log::LevelFilter::Info,
                )
                .filter(
                    Some("catgirl_engine_utils"),
                    tracing::log::LevelFilter::Info,
                );
        }

        builder.try_init().unwrap();
    }
}

/// Setup a hook to catch panics for logging before shutdown
fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location_string = if let Some(location) = info.location() {
            format!(
                " in file {} at line:column {}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        } else {
            String::new()
        };

        if let Some(string) = info.payload().downcast_ref::<String>() {
            error!("Caught panic{location_string}: {string}");
        } else {
            error!("Caught panic{location_string}");
        }

        utils::exit::set_exit();
    }));
}

// /// This functions intentionally triggers a panic
// ///
// /// # Panics
// ///
// /// Always
// fn trigger_panic() {
//     let message: &str = "Intentionally triggered a panic for debugging...";

//     // So, I can't pass a String from to_string(), but can pass as a formatted string
//     panic!("{}", message);
// }

/// Determines if client or server and starts the engine
///
/// # Panics
///
/// This may fail to set the ctrl+c handler
///
/// # Errors
///
/// This may fail to set the ctrl+c handler
pub(super) fn start() -> Result<(), String> {
    info!("Starting Game...");

    debug!("Setting panic hook...");
    set_panic_hook();

    debug!("Starting main loop...");

    #[cfg(feature = "server")]
    if cfg!(not(feature = "client")) || get_args().server {
        // Server exists, client may exist
        return server::game::game_loop();
    }

    // Client exists, server may exist
    #[cfg(feature = "client")]
    return client::game::game_loop();

    // Server doesn't exist, client doesn't exist
    #[cfg(not(feature = "client"))]
    Err("Neither the client nor server features were configured at build time...".to_string())
}
