# invent-monorepo

A Rust backend monorepo for inventory management, built with Axum, SeaORM, and PostgreSQL. Designed with SOLID principles — clean separation between the database layer, business logic, and HTTP handlers.

---

## Table of Contents

- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Local Setup](#local-setup)
- [API Endpoints](#api-endpoints)
- [CI / GitHub Actions](#ci--github-actions)
- [How to Open a Pull Request](#how-to-open-a-pull-request)
- [Developer Guide](#developer-guide)
- [Environment Variables Reference](#environment-variables-reference)

---

## Architecture

```
.
├── crates/
│   ├── app-runner/              # HTTP server bootstrap (Axum + graceful shutdown)
│   └── db/                      # Database layer — entities, repos, base trait
│       └── src/
│           ├── connection.rs    # SeaORM pool setup
│           ├── error.rs         # Unified DbError type
│           ├── repository.rs    # Generic Repository<Model, Create, Update> trait + Pagination
│           ├── entities/        # SeaORM entity definitions (one per table)
│           │   └── product.rs
│           └── repos/           # Concrete repo implementations
│               └── product_repo.rs
│
├── invent-core/                 # Main application — all HTTP and business logic lives here
│   └── src/
│       ├── main.rs              # Bootstrap only: env, DB connection, dependency wiring
│       ├── app.rs               # Router composition — register new domains here
│       ├── domain/              # One sub-folder per business domain
│       │   └── product/
│       │       ├── dto.rs       # Request/response types + validate() methods
│       │       ├── service.rs   # ProductRepo trait, ProductService trait, ProductServiceImpl
│       │       ├── handlers.rs  # Thin HTTP handlers (depend on Arc<dyn ProductService>)
│       │       └── routes.rs    # Route definitions for this domain
│       ├── middleware/
│       │   └── auth.rs          # JWT validation middleware + AuthConfig + UserContext
│       ├── extractors/
│       │   └── auth_user.rs     # AuthUser extractor (pulls UserContext from request)
│       └── shared/
│           ├── errors.rs        # AppError — maps DbError to HTTP responses
│           ├── permissions.rs   # Role-based permission guards
│           └── response.rs      # ApiResponse<T>, PagedData<T>, ok/created/no_content helpers
│
├── traefik/                     # Reverse proxy config for local dev
└── scripts/                     # Dev helper scripts
```

### Dependency flow

```
HTTP request
    │
    ▼
handlers.rs          (parses request, calls service, formats response)
    │ Arc<dyn ProductService>
    ▼
service.rs           (business logic, validation, calls repo)
    │ Arc<dyn ProductRepo>
    ▼
repos/product_repo.rs  (database queries only, SeaORM)
    │
    ▼
PostgreSQL
```

Nothing flows upward. `crates/db` has zero knowledge of HTTP or Axum. Services have zero knowledge of HTTP concepts — they receive plain Rust types and return `Result<T, AppError>`.

### Layered architecture

| Layer | Location | Responsibility |
|---|---|---|
| **API** | `domain/*/handlers.rs` | Parse HTTP request, call service, format response |
| **Service** | `domain/*/service.rs` | Business logic, call repository, return results |
| **Repository** | `crates/db/repos/` | Database queries only — no business logic |
| **Shared** | `shared/` | Cross-cutting: errors, response helpers, permissions |

### Key design decisions

**Dependency Inversion throughout**
Handlers depend on `Arc<dyn ProductService>` (a trait), not the concrete `ProductServiceImpl`. The service depends on `Arc<dyn ProductRepo>` (a trait), not the concrete `ProductRepository`. Concrete types are only named in `main.rs` where the dependency graph is wired. Everything below `main.rs` works against interfaces.

**Validation at the DTO layer**
Each request struct (`CreateProductRequest`, `UpdateProductRequest`) has a `validate()` method that returns `AppError::BadRequest`. Handlers call `payload.validate()?` before touching the service. The service only receives pre-validated data — it never needs to re-check or return validation errors.

**Standardised response envelope**
Every successful response uses `shared::response` helpers (`ok`, `created`, `no_content`). All responses follow the same shape:
```json
{ "success": true, "data": { ... } }
```
Errors are handled automatically by `AppError::into_response` — no manual status code matching in handlers.

**`AppError` bridges `DbError` → HTTP**
`From<DbError> for AppError` is implemented once in `shared/errors.rs`. The `?` operator converts db errors to `AppError` automatically. Handlers never import `DbError`.

**SeaORM over raw sqlx**
No compile-time database connection required. Queries are built at runtime using type-safe query builders. Entities map directly to Postgres tables.

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- A PostgreSQL database (we use [Neon](https://neon.tech) — free tier works)
- Traefik (optional, for local reverse proxy)

---

## Local Setup

### 1. Clone the repo

```bash
git clone https://github.com/dushyantpant5/invent-monorepo.git
cd invent-monorepo
```

### 2. Configure environment

Copy the example env and fill in your values:

```bash
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=postgres://user:password@host/dbname?sslmode=require
DB_MAX_CONNS=5
JWT_SECRET=your-secret-key-here
PORT=8082
```

> For Neon, copy the connection string from your Neon dashboard. It includes SSL by default.

### 3. Load env and build

```bash
set -a && source .env && set +a
cargo build
```

### 4. Run the server

```bash
cargo run -p invent-core
```

The server starts at `http://localhost:8082`.

### 5. (Optional) Run with Traefik

```bash
cargo make run-all
```

This starts both the API and Traefik reverse proxy together and streams their logs.

---

## API Endpoints

All `/api/v1/*` routes require a `Authorization: Bearer <jwt>` header.

### Infrastructure

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Liveness check |
| GET | `/ready` | No | DB connectivity check |

### Products

| Method | Path | Role required | Description |
|--------|------|---------------|-------------|
| POST | `/api/v1/product` | Admin | Create a product |
| GET | `/api/v1/product/:id` | Any | Get a product by ID |
| GET | `/api/v1/product/my` | Any | List the caller's products (paginated) |
| PUT | `/api/v1/product/:id` | Admin, Staff | Update a product |
| DELETE | `/api/v1/product/:id` | Admin | Delete a product |

> **Note on `price`:** always send as a decimal string — `"29.99"`, not `29.99`. Floating-point literals lose precision.

### Example requests

**Create a product**
```bash
curl -X POST http://localhost:8082/api/v1/product \
  -H "Authorization: Bearer <your-jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Widget Pro",
    "description": "A great widget",
    "price": "29.99",
    "quantity": 100,
    "category_id": null
  }'
```

**Get a product**
```bash
curl http://localhost:8082/api/v1/product/<uuid> \
  -H "Authorization: Bearer <your-jwt>"
```

**List your products (paginated)**
```bash
curl "http://localhost:8082/api/v1/product/my?page=1&page_size=20" \
  -H "Authorization: Bearer <your-jwt>"
```

**Update a product**
```bash
curl -X PUT http://localhost:8082/api/v1/product/<uuid> \
  -H "Authorization: Bearer <your-jwt>" \
  -H "Content-Type: application/json" \
  -d '{ "quantity": 50 }'
```

**Delete a product**
```bash
curl -X DELETE http://localhost:8082/api/v1/product/<uuid> \
  -H "Authorization: Bearer <your-jwt>"
```

### Response shape

All successful responses follow this envelope:

```json
{ "success": true, "data": { ... } }
```

Paginated responses:

```json
{
  "success": true,
  "data": {
    "items": [ ... ],
    "total": 42,
    "limit": 20,
    "offset": 0
  }
}
```

Error responses:

```json
{ "error": "description of what went wrong" }
```

---

## CI / GitHub Actions

Two workflows run automatically:

### `pr-check` — runs on every Pull Request

| Step | What it does |
|------|--------------|
| `cargo check` | Fast compile check across the whole workspace |
| `cargo fmt` | Fails if code is not formatted with `rustfmt` |
| `cargo clippy` | Lints for common mistakes — warnings are errors |
| `shellcheck` | Lints `traefik/start-traefik.sh` |

### `build` — runs on push to `develop`

| Step | What it does |
|------|--------------|
| `cargo test` | Runs all workspace tests |
| Traefik smoke | Renders `dynamic.yml.template`, validates YAML, checks shellcheck |

---

## How to Open a Pull Request

### 1. Create a branch from `develop`

Always branch off `develop`, never `main`:

```bash
git checkout develop
git pull origin develop
git checkout -b feat/your-feature-name
```

Branch naming conventions:

- `feat/` — new feature
- `fix/` — bug fix
- `chore/` — tooling, deps, config
- `refactor/` — code restructure, no behaviour change

### 2. Make your changes

Keep changes focused. One PR = one concern.

### 3. Format and lint before pushing

The CI will fail if these do not pass — run them locally first:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check --workspace
```

### 4. Push and open the PR

```bash
git add .
git commit -m "feat: add product search endpoint"
git push origin feat/your-feature-name
```

Then open a PR on GitHub targeting the `develop` branch.

### 5. Wait for `pr-check` to pass

The `pr-check` workflow runs automatically. Fix any fmt or clippy failures before requesting review.

---

## Developer Guide

This section explains the architecture patterns enforced in this codebase. Read this before adding any new code.

### Core principle: every layer has one job

```
Handler   → parse request, check permission, validate, call service, return response
Service   → business logic, call repository, return Result<T, AppError>
Repository → database queries only, return Result<T, DbError>
Shared    → reusable helpers that belong to no single domain
```

Violating this — e.g. writing a database query inside a handler, or reading an HTTP header inside a service — is a bug in the architecture, not just style.

---

### How to add a new domain

Adding a new domain (e.g. `category`) requires creating four files and registering two lines. No existing file needs to change except the two registration points.

#### Step 1 — Add the database entity

`crates/db/src/entities/category.rs`

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

Export from `crates/db/src/entities/mod.rs`:
```rust
pub mod category;
pub use category::Model as Category;
```

#### Step 2 — Add the repository

`crates/db/src/repos/category_repo.rs` — implement `Repository<Category, CreateCategory, UpdateCategory>` following the same pattern as `product_repo.rs`.

Export from `crates/db/src/repos/mod.rs`:
```rust
pub mod category_repo;
pub use category_repo::{CategoryRepository, CreateCategory, UpdateCategory};
```

#### Step 3 — Create the domain module

```
invent-core/src/domain/category/
├── mod.rs
├── dto.rs
├── service.rs
├── handlers.rs
└── routes.rs
```

**`dto.rs`** — request/response types, each with a `validate()` method:
```rust
pub struct CreateCategoryRequest {
    pub name: String,
}

impl CreateCategoryRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequest("name is required".into()));
        }
        Ok(())
    }
}
```

**`service.rs`** — define the repository interface, the service interface, and the concrete implementation:
```rust
// 1. Repository interface — what the service needs from the db
#[async_trait]
pub trait CategoryRepo: Send + Sync + 'static {
    async fn find_by_id(&self, id: Uuid) -> Result<Category, AppError>;
    async fn create(&self, payload: CreateCategory) -> Result<Category, AppError>;
    // ...
}

// 2. Adapter — implement CategoryRepo for the concrete db type
#[async_trait]
impl CategoryRepo for CategoryRepository { ... }

// 3. Service interface — what handlers depend on
#[async_trait]
pub trait CategoryService: Send + Sync + 'static {
    async fn create_category(&self, req: CreateCategoryRequest) -> Result<Category, AppError>;
    // ...
}

// 4. Concrete implementation
pub struct CategoryServiceImpl { repo: Arc<dyn CategoryRepo> }

#[async_trait]
impl CategoryService for CategoryServiceImpl { ... }
```

**`handlers.rs`** — thin handlers that follow the three-step pattern:
```rust
pub async fn create_category(
    State(service): State<Arc<dyn CategoryService>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), AppError> {
    // 1. Authorise
    if !CategoryPermission::can_create(&user) {
        return Err(AppError::Forbidden("not allowed"));
    }
    // 2. Validate
    payload.validate()?;
    // 3. Delegate
    let category = service.create_category(payload).await?;
    Ok(created(CategoryResponse::from(category)))
}
```

**`routes.rs`** — one function, one router:
```rust
pub fn routes(service: Arc<dyn CategoryService>) -> Router {
    Router::new()
        .route("/", post(handlers::create_category))
        .route("/:id", get(handlers::get_category))
        .with_state(service)
}
```

#### Step 4 — Register the domain (two lines)

`invent-core/src/domain/mod.rs`:
```rust
pub mod category;   // ← add this
pub mod product;
```

`invent-core/src/main.rs` — wire deps:
```rust
let category_repo: Arc<dyn CategoryRepo> = Arc::new(CategoryRepository::new(db.clone()));
let category_service: Arc<dyn CategoryService> = Arc::new(CategoryServiceImpl::new(category_repo));
```

`invent-core/src/app.rs` — register routes:
```rust
let api_router = Router::new()
    .nest("/product", product_routes::routes(product_service))
    .nest("/category", category_routes::routes(category_service))  // ← add this
    .layer(...);
```

---

### Handler pattern (reference)

Every handler must follow this exact three-step order. No exceptions.

```rust
pub async fn example_handler(
    State(service): State<Arc<dyn ExampleService>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<ExampleRequest>,
) -> Result<Json<ApiResponse<ExampleResponse>>, AppError> {
    // Step 1: Authorise — reject before doing any work
    if !ExamplePermission::can_do_thing(&user) {
        return Err(AppError::Forbidden("not allowed"));
    }

    // Step 2: Validate — reject invalid input before touching the service
    payload.validate()?;

    // Step 3: Delegate — call service, map result, return
    let result = service.do_thing(payload).await?;
    Ok(ok(ExampleResponse::from(result)))
}
```

Handlers must never:
- Run database queries directly
- Contain `if/else` business logic beyond permission checks
- Know about `DbError` or `SeaORM` types

---

### Service pattern (reference)

Services receive plain Rust types and return `Result<T, AppError>`. They must never:
- Import `axum`, `StatusCode`, `Json`, or any HTTP type
- Read request headers or touch the HTTP request
- Return `DbResult<T>` — always map to `AppError` via `?` or `.map_err(AppError::from)`

```rust
async fn create_thing(&self, user_id: Uuid, req: ThingRequest) -> Result<Thing, AppError> {
    self.repo
        .create(CreateThing {
            user_id,
            name: req.name.trim().to_string(),
        })
        .await  // DbError is automatically converted to AppError via From impl
}
```

---

### Response helpers (reference)

Always use `shared::response` — never write `(StatusCode::CREATED, Json(...))` manually.

```rust
use crate::shared::response::{ok, created, no_content, ApiResponse, PagedData};

// 200 OK
Ok(ok(MyResponse::from(item)))

// 201 Created
Ok(created(MyResponse::from(item)))

// 204 No Content
Ok(no_content())

// 200 OK with pagination
let data = PagedData::new(
    page.items.into_iter().map(MyResponse::from).collect(),
    page.total,
    page.limit,
    page.offset,
);
Ok(ok(data))
```

---

### Adding permissions for a new domain

Add a new struct to `shared/permissions.rs`:

```rust
pub struct CategoryPermission;

impl CategoryPermission {
    pub fn can_create(user: &UserContext) -> bool {
        matches!(user.role, Roles::Admin)
    }
}
```

Do not put permission logic inside handlers or services.

---

### Error handling (reference)

`AppError` in `shared/errors.rs` is the single error type for the entire application.

| Variant | HTTP status | When to use |
|---|---|---|
| `NotFound` | 404 | Resource does not exist |
| `Conflict(msg)` | 409 | Duplicate key, unique constraint violated |
| `BadRequest(msg)` | 400 | Invalid input, failed validation |
| `Forbidden(msg)` | 403 | Caller lacks permission |
| `Unauthorized` | 401 | No valid auth token |
| `Internal` | 500 | Unexpected error — log and hide details |

`DbError` automatically maps to `AppError` via the `From` impl — use `?` and it converts for you.

---

## Environment Variables Reference

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | Full Postgres connection string |
| `DB_MAX_CONNS` | No | Max DB connections (default: 5) |
| `JWT_SECRET` | Yes | Secret used to validate HS256 JWTs |
| `PORT` | No | Port to listen on (default: 8082) |
| `PRODUCT_SERVICE_URL` | No | Used by Traefik routing config |
