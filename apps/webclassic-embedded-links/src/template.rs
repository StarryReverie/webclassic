use minijinja::Environment;

pub fn create_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("list", include_str!("../static/list.template.html"))
        .unwrap();
    env
}
