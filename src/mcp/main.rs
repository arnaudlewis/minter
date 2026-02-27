use rmcp::ServiceExt;
use std::process;

#[tokio::main]
async fn main() {
    let server = minter::mcp::tools::MinterServer::new();
    let service = match server.serve(rmcp::transport::io::stdio()).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to start MCP server on stdio: {e}");
            process::exit(1);
        }
    };
    match service.waiting().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("MCP server exited with error: {e}");
            process::exit(1);
        }
    }
}
