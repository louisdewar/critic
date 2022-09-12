struct MyFixture {
    hello: String,
}

#[critic::fixture]
fn produce() -> MyFixture {
    println!("produced fixture MyFixture");
    MyFixture {
        hello: "world".to_string(),
    }
}

critic::critic_test_main!();

#[critic::test]
fn my_test(my_fixture: &MyFixture) {
    println!("1 hello {}", my_fixture.hello);
    println!(
        "1 type of MyFixture: {}",
        std::any::type_name::<MyFixture>()
    );
}

#[critic::test]
fn my_test_2(my_fixture: &MyFixture) {
    println!("2 hello {}", my_fixture.hello);
    println!(
        "2 type of MyFixture: {}",
        std::any::type_name::<MyFixture>()
    );
}

#[critic::test]
fn my_test_3(my_fixture: &mut MyFixture) {
    my_fixture.hello = "**new** World".to_string();
    println!("Modifed my_fixture.hello");
}

#[critic::test]
fn abc() {
    println!("Hello from ABC");
}

#[critic::test]
fn abc2() {
    println!("Hello from ABC2");
}

mod sub_module {
    use crate::MyFixture;

    // #[critic::test]
    // fn my_test_in_submodule(my_fixture: MyFixture) {
    //     println!("submodule says: {}", my_fixture.hello);
    // }
}
