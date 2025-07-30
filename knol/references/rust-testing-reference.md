# Rust Testing Catalog: Comprehensive Reference Guide

## Table of Contents

1. [Mocking Frameworks and Tools](#1-mocking-frameworks-and-tools)
2. [Spy Techniques and Implementations](#2-spy-techniques-and-implementations)
3. [Dependency Injection Patterns](#3-dependency-injection-patterns)
4. [Best Practices for Testable Rust Code](#4-best-practices-for-testable-rust-code)
5. [Design Patterns for Testability](#5-design-patterns-for-testability)
6. [TDD Workflows and Tools](#6-tdd-workflows-and-tools)
7. [XP Practices for Rust](#7-xp-practices-for-rust)
8. [Testing Libraries Beyond std](#8-testing-libraries-beyond-std)
9. [Property-Based Testing](#9-property-based-testing)
10. [Integration and E2E Testing](#10-integration-and-e2e-testing)
11. [Code Coverage Tools](#11-code-coverage-tools)
12. [Continuous Integration Setups](#12-continuous-integration-setups)
13. [Performance and Benchmark Testing](#13-performance-and-benchmark-testing)
14. [Async/Await Testing Patterns](#14-asyncawait-testing-patterns)
15. [Error Handling for Testability](#15-error-handling-for-testability)
16. [Examples and Code Snippets](#16-examples-and-code-snippets)

---

## 1. Mocking Frameworks and Tools

### **Taxonomy of Rust Mocking Solutions**

#### 1.1 Trait-Based Mocking

**Mockall** - The Gold Standard
- **Category**: Compile-time safe mocking
- **Use Case**: Complex trait mocking with strong typing
- **Ownership Model**: Excellent compatibility

```rust
use mockall::*;

#[automock]
trait Database {
    fn get_user(&self, id: u32) -> Result<User, Error>;
    fn save_user(&mut self, user: &User) -> Result<(), Error>;
}

#[test]
fn test_with_mockall() {
    let mut mock = MockDatabase::new();

    mock.expect_get_user()
        .with(predicate::eq(42))
        .times(1)
        .returning(|_| Ok(User { id: 42, name: "Test".into() }));

    let result = mock.get_user(42);
    assert!(result.is_ok());
}
```

#### 1.2 Struct-Based Mocking

**Faux** - Traitless Mocking
- **Category**: Struct method mocking
- **Use Case**: When you don't want to create traits
- **Ownership Model**: Good, with some unsafe internals

```rust
#[faux::create]
pub struct UserService {
    // fields
}

#[faux::methods]
impl UserService {
    pub fn fetch_user(&self, id: u32) -> User {
        // implementation
    }
}

#[test]
fn test_with_faux() {
    let mut service = UserService::faux();
    when!(service.fetch_user).given(42).then_return(User::test());
}
```

#### 1.3 HTTP Mocking

**Wiremock** - HTTP Service Mocking
- **Category**: Integration test mocking
- **Use Case**: Testing HTTP clients
- **Ownership Model**: Async-first design

```rust
#[tokio::test]
async fn test_http_client() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/users/123"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"id": 123, "name": "Test User"})))
        .mount(&mock_server)
        .await;

    let client = ApiClient::new(&mock_server.uri());
    let user = client.get_user(123).await.unwrap();
    assert_eq!(user.name, "Test User");
}
```

---

## 2. Spy Techniques and Implementations

### **Taxonomy of Spy Patterns**

#### 2.1 Function Spies

**spy crate** - JavaScript-inspired spying
```rust
use spy::spy;

#[test]
fn test_callback_spy() {
    let (spy_fn, spy) = spy!(|x: i32| x * 2);

    let result = vec![1, 2, 3].iter().map(spy_fn).collect::<Vec<_>>();

    let snapshot = spy.snapshot();
    assert_eq!(snapshot.num_calls(), 3);
    assert_eq!(snapshot.first_call().unwrap(), &1);
}
```

#### 2.2 Custom Spy Implementation Pattern
```rust
struct CallRecorder<T> {
    calls: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone> CallRecorder<T> {
    fn record(&self, value: T) {
        self.calls.lock().unwrap().push(value);
    }

    fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}
```

---

## 3. Dependency Injection Patterns

### **Taxonomy of DI Approaches**

#### 3.1 Constructor Injection with Generics

**Zero-Cost Abstraction Pattern**
```rust
trait Logger {
    fn log(&self, message: &str);
}

trait Database {
    fn query(&self, sql: &str) -> Result<Vec<Row>, Error>;
}

struct Service<L: Logger, D: Database> {
    logger: L,
    database: D,
}

impl<L: Logger, D: Database> Service<L, D> {
    fn new(logger: L, database: D) -> Self {
        Self { logger, database }
    }

    fn process(&self) -> Result<(), Error> {
        self.logger.log("Processing started");
        let data = self.database.query("SELECT * FROM users")?;
        // Process data
        Ok(())
    }
}
```

#### 3.2 The Deps Pattern

**Function-Level DI**
```rust
fn process_data(deps: &(impl Logger + Database)) -> Result<(), Error> {
    deps.log("Processing");
    let data = deps.query("SELECT * FROM data")?;
    Ok(())
}
```

#### 3.3 Newtype Pattern for Encapsulation

```rust
pub struct Connection(Arc<dyn Database + Send + Sync + 'static>);

impl Connection {
    pub fn postgres(config: Config) -> Self {
        Self(Arc::new(PostgresDb::new(config)))
    }

    pub fn test() -> Self {
        Self(Arc::new(InMemoryDb::new()))
    }
}
```

---

## 4. Best Practices for Testable Rust Code

### **Core Principles**

#### 4.1 Trait-Based Abstraction
```rust
// Good: Trait abstraction
trait PaymentProcessor {
    fn charge(&self, amount: Money) -> Result<Receipt, PaymentError>;
}

// Bad: Concrete dependency
struct OrderService {
    stripe_client: StripeClient, // Hard to test!
}
```

#### 4.2 Explicit Dependencies
```rust
// Good: All dependencies visible
fn calculate_discount<P: PricingService, U: UserService>(
    pricing: &P,
    users: &U,
    user_id: u32,
) -> Result<Discount, Error> {
    let user = users.get_user(user_id)?;
    pricing.calculate_discount(&user)
}
```

#### 4.3 Pure Functions Where Possible
```rust
// Good: Pure function, easy to test
fn calculate_tax(subtotal: Money, tax_rate: f64) -> Money {
    subtotal * tax_rate
}

// Less testable: Side effects mixed with logic
fn process_order(&mut self, order: Order) -> Result<(), Error> {
    let tax = order.subtotal * self.get_tax_rate()?; // Network call
    self.save_to_db(order)?; // Database side effect
    Ok(())
}
```

---

## 5. Design Patterns for Testability

### **Structural Patterns**

#### 5.1 Repository Pattern
```rust
trait UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<Option<User>, Error>;
    fn save(&self, user: &User) -> Result<(), Error>;
    fn delete(&self, id: UserId) -> Result<(), Error>;
}

// Production implementation
struct PostgresUserRepository {
    pool: PgPool,
}

// Test implementation
struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<UserId, User>>>,
}
```

#### 5.2 Factory Pattern with Traits
```rust
trait ConnectionFactory {
    fn create(&self) -> Result<Box<dyn Connection>, Error>;
}

struct MultiDatabaseFactory {
    config: DatabaseConfig,
}

impl ConnectionFactory for MultiDatabaseFactory {
    fn create(&self) -> Result<Box<dyn Connection>, Error> {
        match self.config.db_type {
            DbType::Postgres => Ok(Box::new(PostgresConnection::new(&self.config)?)),
            DbType::SQLite => Ok(Box::new(SqliteConnection::new(&self.config)?)),
        }
    }
}
```

#### 5.3 Strategy Pattern
```rust
trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}

struct FileProcessor<C: CompressionStrategy> {
    compression: C,
}

impl<C: CompressionStrategy> FileProcessor<C> {
    fn process_file(&self, path: &Path) -> Result<(), Error> {
        let data = std::fs::read(path)?;
        let compressed = self.compression.compress(&data);
        // Save compressed data
        Ok(())
    }
}
```

---

## 6. TDD Workflows and Tools

### **Red-Green-Refactor in Rust**

#### 6.1 Essential TDD Tools

**cargo-watch** - Continuous Testing
```bash
# Install
cargo install cargo-watch

# Run tests on file change
cargo watch -x test

# Run check, then test
cargo watch -x check -x test
```

#### 6.2 TDD Workflow Example
```rust
// Step 1: Write failing test (RED)
#[test]
fn test_calculate_fibonacci() {
    assert_eq!(fibonacci(0), 0);
    assert_eq!(fibonacci(1), 1);
    assert_eq!(fibonacci(5), 5);
}

// Step 2: Make it pass (GREEN)
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Step 3: Refactor
fn fibonacci(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;

    for _ in 0..n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    a
}
```

---

## 7. XP Practices for Rust

### **Extreme Programming Adaptations**

#### 7.1 Pair Programming Setup
- VS Code Live Share
- IntelliJ Rust with Code With Me
- tmux + vim for terminal enthusiasts

#### 7.2 Continuous Integration
```yaml
# GitHub Actions XP setup
name: XP Workflow
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    steps:
      - uses: actions/checkout@v4
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

#### 7.3 Small Releases
```toml
# Cargo.toml - Feature flags for incremental releases
[features]
default = ["stable-features"]
stable-features = []
experimental = ["new-api", "performance-optimizations"]
new-api = []
performance-optimizations = []
```

---

## 8. Testing Libraries Beyond std

### **Enhanced Testing Frameworks**

#### 8.1 rstest - Fixture-Based Testing
```rust
use rstest::*;

#[fixture]
fn database() -> TestDatabase {
    TestDatabase::new()
}

#[rstest]
fn test_user_creation(database: TestDatabase) {
    let user = database.create_user("test@example.com");
    assert!(user.is_ok());
}

#[rstest]
#[case(2, 2, 4)]
#[case(3, 3, 9)]
#[case(4, 4, 16)]
fn test_multiplication(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
    assert_eq!(a * b, expected);
}
```

#### 8.2 test-case - Parameterized Testing
```rust
use test_case::test_case;

#[test_case(-2, -4 ; "negative numbers")]
#[test_case(2, 4 ; "positive numbers")]
#[test_case(0, 0 ; "zeros")]
fn test_addition(a: i32, b: i32) {
    assert_eq!(a + b, a + b); // Tautology for example
}
```

---

## 9. Property-Based Testing

### **Automated Test Generation**

#### 9.1 proptest - Hypothesis-Style Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_vec_reverse_twice(vec: Vec<i32>) {
        let reversed_twice = vec.iter()
            .copied()
            .rev()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        prop_assert_eq!(vec, reversed_twice);
    }
}

// Custom strategies
fn email_strategy() -> impl Strategy<Value = String> {
    "[a-z]+@[a-z]+\\.[a-z]+"
        .prop_map(|s| s.to_string())
}
```

#### 9.2 quickcheck - Type-Based Generation
```rust
use quickcheck::{quickcheck, Arbitrary, Gen};

#[derive(Clone, Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Arbitrary for Rectangle {
    fn arbitrary(g: &mut Gen) -> Self {
        Rectangle {
            width: u32::arbitrary(g) % 100 + 1,
            height: u32::arbitrary(g) % 100 + 1,
        }
    }
}

quickcheck! {
    fn area_is_positive(rect: Rectangle) -> bool {
        rect.width * rect.height > 0
    }
}
```

---

## 10. Integration and E2E Testing

### **Multi-Layer Testing Strategies**

#### 10.1 Integration Test Organization
```
tests/
├── common/
│   └── mod.rs      # Shared test utilities
├── api_tests.rs    # API integration tests
├── db_tests.rs     # Database integration tests
└── e2e_tests.rs    # End-to-end tests
```

#### 10.2 Database Integration Testing
```rust
// tests/db_tests.rs
use sqlx::PgPool;

#[sqlx::test]
async fn test_user_crud(pool: PgPool) {
    let user_id = sqlx::query_scalar!(
        "INSERT INTO users (email) VALUES ($1) RETURNING id",
        "test@example.com"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(user_id > 0);
}
```

#### 10.3 E2E Web Service Testing
```rust
#[tokio::test]
async fn test_full_user_journey() {
    let app = test_app().await;

    // Register
    let response = app.client
        .post("/api/register")
        .json(&json!({
            "email": "user@example.com",
            "password": "secure123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Login
    let login_response = app.client
        .post("/api/login")
        .json(&json!({
            "email": "user@example.com",
            "password": "secure123"
        }))
        .send()
        .await
        .unwrap();

    let token: LoginResponse = login_response.json().await.unwrap();
    assert!(!token.access_token.is_empty());
}
```

---

## 11. Code Coverage Tools

### **Coverage Analysis Solutions**

#### 11.1 Tool Comparison Matrix

| Tool | Platform Support | Accuracy | CI Integration | Setup Complexity |
|------|-----------------|----------|----------------|------------------|
| **tarpaulin** | Linux native, Docker others | Good | Excellent | Low |
| **grcov** | All platforms | Excellent | Excellent | Medium |
| **llvm-cov** | All platforms | Excellent | Good | Low |

#### 11.2 Tarpaulin Setup
```bash
# Install
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# CI integration
cargo tarpaulin --out Xml
curl -s https://codecov.io/bash | bash
```

#### 11.3 grcov Configuration
```bash
# Setup
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"

# Run tests
cargo test

# Generate report
grcov . --binary-path ./target/debug/ -s . -t html \
    --branch --ignore-not-existing -o ./coverage/
```

---

## 12. Continuous Integration Setups

### **CI/CD Configurations**

#### 12.1 GitHub Actions Matrix Testing
```yaml
name: Rust CI
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test --all-features
```

#### 12.2 GitLab CI with Coverage
```yaml
stages:
  - test
  - coverage
  - deploy

test:cargo:
  stage: test
  image: rust:latest
  script:
    - cargo test --workspace --all-features

coverage:
  stage: coverage
  image: rust:latest
  script:
    - cargo install grcov
    - export RUSTFLAGS="-Cinstrument-coverage"
    - export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"
    - cargo test
    - grcov . --binary-path ./target/debug/ -t cobertura -o coverage.xml
  coverage: '/Coverage: \d+\.\d+%/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.xml
```

---

## 13. Performance and Benchmark Testing

### **Benchmarking Tools**

#### 13.1 Criterion.rs - Statistical Benchmarking
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    for i in [20, 25, 30].iter() {
        group.bench_with_input(format!("fib {}", i), i, |b, i| {
            b.iter(|| fibonacci(black_box(*i)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

#### 13.2 Built-in bench (Nightly)
```rust
#![feature(test)]
extern crate test;

#[bench]
fn bench_vec_push(b: &mut test::Bencher) {
    b.iter(|| {
        let mut vec = Vec::new();
        for i in 0..1000 {
            vec.push(i);
        }
    });
}
```

---

## 14. Async/Await Testing Patterns

### **Asynchronous Test Strategies**

#### 14.1 Tokio Testing
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert_eq!(result, expected);
}

#[tokio::test(start_paused = true)]
async fn test_with_time_control() {
    let start = tokio::time::Instant::now();
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Time is paused, so this completes instantly
    assert!(start.elapsed() < Duration::from_millis(10));
}
```

#### 14.2 Mock Async I/O
```rust
use tokio_test::io::Builder;

async fn echo_server<R, W>(reader: R, writer: W)
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    // Implementation
}

#[tokio::test]
async fn test_echo() {
    let reader = Builder::new()
        .read(b"hello\n")
        .build();

    let writer = Builder::new()
        .write(b"hello\n")
        .build();

    echo_server(reader, writer).await;
}
```

#### 14.3 Async Concurrency Testing
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let results = tokio::join!(
        async_operation_1(),
        async_operation_2(),
        async_operation_3(),
    );

    assert!(results.0.is_ok());
    assert!(results.1.is_ok());
    assert!(results.2.is_ok());
}
```

---

## 15. Error Handling for Testability

### **Testable Error Design**

#### 15.1 Custom Error Types
```rust
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Resource not found: {id}")]
    NotFound { id: u32 },
}

// Easy to test specific errors
#[test]
fn test_error_conditions() {
    let result = service.get_user(0);
    assert!(matches!(
        result,
        Err(ServiceError::Validation(_))
    ));
}
```

#### 15.2 Result Type Testing
```rust
fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

#[test]
fn test_division() {
    assert_eq!(divide(10.0, 2.0), Ok(5.0));
    assert_eq!(divide(10.0, 0.0), Err(MathError::DivisionByZero));
}
```

---

## 16. Examples and Code Snippets

### **Complete Testing Examples**

#### 16.1 Full Service Test Suite
```rust
mod tests {
    use super::*;
    use mockall::predicate::*;
    use rstest::*;

    #[fixture]
    fn mock_db() -> MockDatabase {
        MockDatabase::new()
    }

    #[fixture]
    fn test_user() -> User {
        User {
            id: 1,
            email: "test@example.com".into(),
            name: "Test User".into(),
        }
    }

    #[rstest]
    fn test_user_service_create(
        mut mock_db: MockDatabase,
        test_user: User,
    ) {
        mock_db
            .expect_save_user()
            .with(eq(test_user.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = UserService::new(mock_db);
        let result = service.create_user(test_user);

        assert!(result.is_ok());
    }

    #[rstest]
    #[case("", "Email cannot be empty")]
    #[case("invalid", "Invalid email format")]
    fn test_email_validation(
        #[case] email: &str,
        #[case] expected_error: &str,
    ) {
        let result = validate_email(email);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), expected_error);
    }

    #[proptest]
    fn test_user_serialization(user: User) {
        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(user, deserialized);
    }
}
```

#### 16.2 Integration Test Example
```rust
// tests/integration/api_test.rs
use crate::common::*;

#[tokio::test]
async fn test_complete_user_flow() {
    let app = spawn_app().await;

    // Create user
    let create_response = app.create_user("test@example.com").await;
    assert_eq!(create_response.status(), 201);

    // Fetch user
    let user = app.get_user(1).await;
    assert_eq!(user.email, "test@example.com");

    // Update user
    let update_response = app.update_user(1, "new@example.com").await;
    assert_eq!(update_response.status(), 200);

    // Verify update
    let updated_user = app.get_user(1).await;
    assert_eq!(updated_user.email, "new@example.com");
}
```

---

## Summary and Best Practices

### **Key Takeaways**

1. **Use the Right Tool for the Job**
   - Mockall for complex trait mocking
   - Proptest for finding edge cases
   - Criterion for performance testing
   - Wiremock for HTTP mocking

2. **Leverage Rust's Type System**
   - Use traits for abstraction
   - Prefer constructor injection
   - Design with testability in mind

3. **Adopt Modern Testing Practices**
   - Use cargo-nextest for faster test runs
   - Implement property-based testing
   - Set up comprehensive CI/CD

4. **Follow Rust-Specific Patterns**
   - Embrace the ownership model in tests
   - Use the type system for test safety
   - Leverage compiler-driven development

This comprehensive guide provides a complete reference for unit testing, XP, and TDD in Rust, with practical examples and clear guidance on when to use each tool and technique.
