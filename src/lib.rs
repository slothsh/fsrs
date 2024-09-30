mod route;

#[cfg(test)]
mod tests {
    use crate::route::server::Route;

    #[test]
    fn server_route_from_file() {
        let route = Route::from_file(std::path::Path::new("./route.rs"))
            .unwrap();

        assert_eq!(4, route.props.len());

        route.write_js_declarations(std::path::Path::new("./foo.ts"));
        route.write_rust_module(std::path::Path::new("./bar.rs"));
    }
}
