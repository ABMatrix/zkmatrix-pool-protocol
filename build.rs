fn main() {
    use vergen::{vergen, Config};

    let config = Config::default();
    vergen(config).unwrap()
}
