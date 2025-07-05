// mcp_server/src/main.rs

use mcpadawan::*;
use clap::Parser;
use std::fs;
use std::path::Path;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
#[cfg(feature = "vigem")]
use vigem_client::{Client, TargetId, Target};

// Command line arguments
#[derive(Parser)]
#[command(name = "mcpadawan")]
#[command(about = "A remote control server for applications")]
struct Args {
    /// Disable ViGEm gamepad integration
    #[arg(long, help = "Disable ViGEm gamepad integration")]
    disable_vigem: bool,
    
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml", help = "Path to configuration file")]
    config: String,
    
    /// Run as MCP server instead of HTTP server
    #[arg(long, help = "Run as MCP (Model Context Protocol) server")]
    mcp: bool,
    
    /// Port for HTTP server
    #[arg(short, long, default_value = "5050", help = "Port for HTTP server")]
    port: u16,
}

// Configuration structure
#[derive(serde::Deserialize, Default)]
struct Config {
    #[serde(default)]
    disable_vigem: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // Load configuration file
    let config: Config = if Path::new(&args.config).exists() {
        let config_content = fs::read_to_string(&args.config)
            .expect("Failed to read configuration file");
        toml::from_str(&config_content)
            .expect("Failed to parse configuration file")
    } else {
        Config::default()
    };
    
    // Set MCP mode flag early so logging works correctly
    if args.mcp {
        mcpadawan::set_mcp_mode(true);
    }
    
    // Determine if ViGEm should be disabled (CLI flag takes precedence)
    let vigem_disabled = args.disable_vigem || config.disable_vigem;
    
    // Initialize ViGEm controller if not disabled
    #[cfg(feature = "vigem")]
    let controller = if vigem_disabled {
        mcpadawan::log_info("ViGEm gamepad integration disabled");
        None
    } else {
        match Client::connect() {
            Ok(client) => {
                let target = Target::new(TargetId::X360);
                match client.plugin(&target) {
                    Ok(_) => {
                        mcpadawan::log_info("ViGEm gamepad integration enabled");
                        Some(target)
                    }
                    Err(e) => {
                        mcpadawan::log_info(&format!("Failed to plug in virtual controller: {}", e));
                        None
                    }
                }
            }
            Err(e) => {
                mcpadawan::log_info(&format!("Failed to connect to ViGEmBus: {}", e));
                None
            }
        }
    };
    
    #[cfg(not(feature = "vigem"))]
    let _controller = {
        if !vigem_disabled {
            mcpadawan::log_info("ViGEm gamepad integration not compiled in (missing 'vigem' feature)");
        } else {
            mcpadawan::log_info("ViGEm gamepad integration disabled");
        }
    };

    let state = AppState {
        enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
        app_process: Arc::new(Mutex::new(None)),
        #[cfg(feature = "vigem")]
        controller: Arc::new(Mutex::new(controller)),
        vigem_enabled: !vigem_disabled && cfg!(feature = "vigem"),
        app_pid: Arc::new(Mutex::new(None)),
    };

    if args.mcp {
        // Run as MCP server
        mcpadawan::log_info("Starting MCPadawan as MCP server...");
        let mcp_server = mcpadawan::mcp::McpServer::new(state);
        if let Err(e) = mcp_server.run().await {
            eprintln!("MCP server error: {}", e);
        }
    } else {
        // Run as HTTP server
        let app = create_app(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                mcpadawan::log_info(&format!("Listening on http://{}", addr));
                if let Err(e) = axum::serve(listener, app).await {
                    eprintln!("HTTP server error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to bind to {}: {}", addr, e);
                std::process::exit(1);
            }
        }
    }
}

