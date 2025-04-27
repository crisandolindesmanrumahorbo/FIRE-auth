Integration test instead unit testing and E2E

- error often occured when in between code (integration test)
- unit testing become important when the a function have a complex and alot edge cases
- E2E: why u testing library in the first place

Using In-Memory Database is more efficient rather than mock the db or actual database.

```
Integration Test
implementing connection pools, each unit depended to each other because using 1 DB.
```

```
install_default_drivers
https://docs.rs/sqlx/latest/sqlx/any/index.html
```

cargo feature -> is like toogle feature
since we put test-sqlite as feature, if we want execute the feature we must put the feature in command line

```
cargo test --features test-sqlite
```
