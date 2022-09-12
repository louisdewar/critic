struct AuthenticationService {}

#[critic::fixture]
fn authentication_service() -> AuthenticationService {
    todo!();
}

// Expands to:
// impl Fixture for AuthenticationService {
//     fn produce() -> Self {
//          todo!();
//     }
// }

#[critic::test]
fn my_test() {}

mod tests_group {
    #[critic::lifecycle]
    fn before_all() {}

    #[critic::lifecycle]
    fn before_each() {}

    #[critic::test]
    #[excludes("my_key")]
    fn my_test_1() {}

    #[critic::test]
    fn my_test_2() {}

    #[critic::test]
    #[excludes("my_key")]
    fn my_test_3() {}

    #[critic::lifecycle]
    fn after_each() {}

    #[critic::lifecycle]
    fn after_all() {}
}
