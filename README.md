# invent-monorepo

A Rust backend monorepo for inventory management, built with Axum, SeaORM, and PostgreSQL. Designed with SOLID principles — clean separation between the database layer, business logic, and HTTP handlers.

---

## Architecture

```
.
├── crates/
│   ├── app-runner/          # HTTP server bootstrap (Axum + graceful shutdown)
│   └── db/                  # Database layer — entities, repos, base trait
│       ├── src/
│       │   ├── connection.rs      # SeaORM pool setup
│       │   ├── error.rs           # Unified DbError type
│       │   ├── repository.rs      # Base Repository<Model, Create, Update> trait
│       │   ├── entities/          # SeaORM entity definitions (one per table)
│       │   │   └── product.rs
│       │   └── repos/             # Concrete repo implementations
│       │       └── product_repo.rs
│
├── invent-core/             # Main application — routes, middleware, services
│   └── src/
│       ├── main.rs                # Wiring: DB → repos → services → routes
│       ├── errors.rs              # AppError: maps DbError → HTTP responses
│       ├── middleware/
│       │   └── auth.rs            # JWT validation middleware
│       ├── extractors/
│       │   └── auth_user.rs       # AuthUser extractor (reads UserContext from request)
│       ├── common/
│       │   └── permissions.rs     # Role-based permission checks
│       └── services/
│           └── product/
│               ├── service.rs     # Business logic + validation
│               └── api/
│                   ├── mod.rs     # Route registration
│                   └── handlers.rs # HTTP handlers (request in, response out)
│
├── traefik/                 # Reverse proxy config for local dev
└── scripts/                 # Dev helper scripts
```

### Dependency flow

```
app-runner  ←  invent-core  ←  crates/db  ←  sea-orm / tokio
```

Nothing flows upward. `crates/db` has zero knowledge of HTTP or Axum. `invent-core` depends on `db` traits, not concrete structs — so repos are swappable and testable in isolation.

### Key design decisions

**SeaORM over raw sqlx** — No compile-time database connection required. Queries are built at runtime using type-safe query builders. Entities map directly to your Postgres tables.

**Base `Repository<Model, Create, Update>` trait** — Every repo implements the same CRUD contract. Services depend on this trait, not concrete types (Dependency Inversion).

**`AppError` bridges `DbError` → HTTP** — Handlers return `Result<_, AppError>`. Axum calls `into_response()` automatically. No manual status code matching scattered across handlers.

**`ProductService` owns all validation** — Name empty? Price negative? Rejected before touching the DB. Handlers are thin: parse request, call service, return response.

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

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Liveness check |
| GET | `/ready` | No | DB connectivity check |
| POST | `/api/v1/product/` | Bearer JWT (Admin) | Create a product |

### Create Product — example request

```bash
curl -X POST http://localhost:8082/api/v1/product/ \
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

> **Note:** `price` must be a decimal string or number — never a float. `"29.99"` is correct, `29.99` may lose precision.

---

## CI / GitHub Actions

Two workflows run automatically:

### `pr-check` — runs on every Pull Request

| Step | What it does |
|------|--------------|
| `cargo check` | Fast compile check across the whole workspace |
| `cargo fmt` | Fails if code isn't formatted with `rustfmt` |
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

The CI will fail if these don't pass — run them locally first:

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

The `pr-check` workflow runs automatically. Fix any fmt or clippy failures before requesting review. The check typically takes under 2 minutes thanks to the cargo cache.

---

## Adding a New Service (e.g. `category`)

Follow this pattern every time:

1. **Add entity** — `crates/db/src/entities/category.rs` with `#[derive(DeriveEntityModel)]`
2. **Add repo** — `crates/db/src/repos/category_repo.rs` implementing `Repository<Category, CreateCategory, UpdateCategory>`
3. **Export** — add to `crates/db/src/entities/mod.rs` and `crates/db/src/repos/mod.rs`
4. **Add service** — `invent-core/src/services/category/service.rs` with business logic
5. **Add handlers** — `invent-core/src/services/category/api/handlers.rs`
6. **Register routes** — wire up in `main.rs` same as `product`

No changes needed to `crates/db/src/repository.rs` or any existing service.

---

## Environment Variables Reference

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | Full Postgres connection string |
| `DB_MAX_CONNS` | No | Max DB connections (default: 5) |
| `JWT_SECRET` | Yes | Secret used to validate HS256 JWTs |
| `PORT` | No | Port to listen on (default: 8082) |
| `PRODUCT_SERVICE_URL` | No | Used by Traefik routing config |
