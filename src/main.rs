use rmcp::{ServiceExt, transport::stdio};
use std::path::PathBuf;
use turnout::{
    catalog::{MAX_SOURCES, SourceConfig},
    mcp::InspectionServer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut paths = Vec::new();
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--library-source" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--library-source requires a path"))?;
            if value.is_empty() {
                anyhow::bail!("--library-source requires a non-empty path");
            }
            paths.push(PathBuf::from(value));
        } else {
            anyhow::bail!("Unknown argument: {}", PathBuf::from(arg).display());
        }
    }
    if paths.len() > MAX_SOURCES {
        anyhow::bail!("At most {MAX_SOURCES} --library-source arguments are allowed");
    }
    let config = SourceConfig {
        paths,
        cwd: std::env::current_dir()?,
    };
    let service = InspectionServer::new(config).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
