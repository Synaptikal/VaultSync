use std::error::Error;
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;
use vaultsync::api::ApiDoc;

fn main() -> Result<(), Box<dyn Error>> {
    let spec = ApiDoc::openapi().to_pretty_json()?;
    let mut file = File::create("docs/openapi.json")?;
    file.write_all(spec.as_bytes())?;
    println!("OpenAPI spec exported to docs/openapi.json");
    Ok(())
}
