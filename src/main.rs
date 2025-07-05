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
    
    // Determine if ViGEm should be disabled (CLI flag takes precedence)
    let vigem_disabled = args.disable_vigem || config.disable_vigem;
    
    // Initialize ViGEm controller if not disabled
    #[cfg(feature = "vigem")]
    let controller = if vigem_disabled {
        println!("ViGEm gamepad integration disabled");
        None
    } else {
        match Client::connect() {
            Ok(client) => {
                let target = Target::new(TargetId::X360);
                match client.plugin(&target) {
                    Ok(_) => {
                        println!("ViGEm gamepad integration enabled");
                        Some(target)
                    }
                    Err(e) => {
                        println!("Failed to plug in virtual controller: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to ViGEmBus: {}", e);
                None
            }
        }
    };
    
    #[cfg(not(feature = "vigem"))]
    let _controller = {
        if !vigem_disabled {
            println!("ViGEm gamepad integration not compiled in (missing 'vigem' feature)");
        } else {
            println!("ViGEm gamepad integration disabled");
        }
    };

    let state = AppState {
        enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
        app_process: Arc::new(Mutex::new(None)),
        #[cfg(feature = "vigem")]
        controller: Arc::new(Mutex::new(controller)),
        vigem_enabled: !vigem_disabled && cfg!(feature = "vigem"),
    };

    if args.mcp {
        // Run as MCP server
        println!("Starting MCPadawan as MCP server...");
        let mcp_server = mcpadawan::mcp::McpServer::new(state);
        if let Err(e) = mcp_server.run().await {
            eprintln!("MCP server error: {}", e);
        }
    } else {
        // Run as HTTP server
        let app = create_app(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
        println!("Listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

