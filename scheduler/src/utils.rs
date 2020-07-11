pub fn get_env(env: &str, default: String) -> String {
	dotenv::var(env).unwrap_or_else(|_| default)
}