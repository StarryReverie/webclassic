use std::error::Error;
use std::path::Path;

use minijinja::Environment;

pub fn create_env(static_dir: &Path) -> Result<Environment<'static>, Box<dyn Error + Send + Sync>> {
    let mut env = Environment::new();
    env.add_template_owned("list", load_template(static_dir)?)?;
    Ok(env)
}

fn load_template(static_dir: &Path) -> Result<String, Box<dyn Error + Send + Sync>> {
    let path = static_dir.join("list.template.html");
    let source = std::fs::read_to_string(&path)?;
    Ok(source)
}
