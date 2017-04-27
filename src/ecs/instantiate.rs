use toml;

trait Instantiatable<C> {
    fn instantiate(value: toml::value::Table) -> C;
}
