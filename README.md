# Music Licensing Workflow

A fullstack application for managing music licensing negotiations between movie production teams and rights holders (labels and independent artists).

Movie teams place songs into scenes, submit license requests, and negotiate terms (fee, territory, media rights, exclusivity) through a structured offer/counter-offer workflow. Rights holders review incoming requests and respond in real time via Server-Sent Events.

## Quick Start

```bash
cp .env.example .env
docker compose up --build
```

- **Frontend:** http://localhost:3000
- **Backend API:** http://localhost:8080/api
- **Swagger UI:** http://localhost:8080/docs

The backend runs migrations automatically on startup.

## Architecture

```
frontend/          React SPA (Vite + TypeScript + Tailwind v4)
backend/           Rust REST API (Actix-web + SQLx + PostgreSQL)
compose.yml        Docker Compose orchestration
```

### Backend

**Rust + Actix-web + SQLx** — chosen for type safety across the full stack (compile-time SQL query validation, exhaustive pattern matching on state machines, zero-cost abstractions for SSE streaming). The domain is inherently stateful (negotiation workflows with complex transition rules), and Rust's type system prevents entire classes of invalid state bugs at compile time.

The backend is organized into domain modules, each with its own model, service, repository, API layer, and error types:

| Module | Responsibility |
|--------|---------------|
| `iam` | Authentication (JWT + refresh tokens), authorization (scope-based RBAC), user management |
| `movie` | Movies and movie team membership (Owner/Supervisor/Editor/Viewer roles) |
| `scene` | Scenes within movies (timecoded segments) |
| `song` | Song catalog, artist and label association |
| `track` | Song placements within scenes (usage type, time range) |
| `label` | Labels, label membership (Owner/Rep/Artist roles) |
| `license` | License requests, offer/counter-offer negotiation, SSE event streaming |
| `kernel` | Shared types (typed IDs, pagination) |
| `error` | Structured error responses with codes, types, and detail maps |

**48 REST endpoints** documented with OpenAPI (utoipa) and served via Swagger UI at `/docs`.

### Frontend

**React 19 + TypeScript + Vite + Tailwind v4 + shadcn/ui** (Base UI primitives).

The frontend ships two complete workspaces:

**Studio Workspace** (movie production team):
- Movie list with search, sort, and creation
- Movie detail with scenes, team members, and licensing progress
- Scene detail with track placements, song association via searchable combobox
- License request creation, submission, counter-offer, and tracking
- License list with status filters

**Rights Holder Workspace** (labels and independent artists):
- Catalog management (add/edit songs, view placements)
- Incoming license request inbox with filters (needs response, status)
- License negotiation: accept, counter-offer (fee, territory, media rights, exclusivity), reject with reason
- Label member management (add/remove members, role assignment)
- Four switchable personas: Label Owner, Label Rep, Label Artist, Independent Artist

Both workspaces include:
- Real-time updates via SSE (license status changes appear instantly)
- Responsive layout (mobile drawer, desktop sidebar)
- Dark/light theme toggle
- Notification bell for license events
- Workspace switcher in user menu

**Dual API mode:** The frontend can run against the real backend (`/api` proxy) or an in-memory mock backend (default for development). Toggle via the API mode selector — no backend required to explore the full UI.

### Data Model

```
users ──┬── user_roles ──── roles (Admin, Producer, Artist, Label Manager, Viewer)
        ├── movie_members ── movies ── scenes ── tracks ── license_requests ── license_offers
        └── label_members ── labels ── songs ─────────────────┘
```

Key relationships:
- A **movie** has **scenes** (timecoded segments), each with **track placements**
- A **track** links a **song** to a scene with usage type and time range
- Each track can have one **license request**, which progresses through: `DRAFT → REQUESTED → APPROVED/REJECTED/CANCELLED`
- Negotiation happens through **license offers** — each offer has a number, side (movie team or rights holder), fee, territory, media rights, exclusivity, and date range
- **Songs** belong to an **artist** (user) and optionally a **label**
- **Labels** have members with roles: Owner (full control), Rep (can negotiate), Artist (read-only on negotiations)
- **Independent artists** (songs with no label) negotiate directly

### Real-Time: Server-Sent Events

SSE is implemented end-to-end, not just suggested:

- **Backend:** `GET /api/licenses/events` streams `LicenseEvent` payloads (submitted, counter_offer, accepted, rejected, cancelled) via `tokio::sync::broadcast`
- **Frontend:** `EventSource` client subscribes on mount, updates UI state on each event
- **Compression fix:** `Content-Encoding: identity` header prevents gzip buffering from delaying SSE delivery to browsers

The SSE stream carries structured events with license ID, track ID, event kind, actor, and timestamp — enough for any client to update its view without re-fetching.

## API Overview

Authentication uses JWT bearer tokens. Register, login, and use the access token:

```bash
# Register
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "producer@acme.com", "password": "Password1!", "name": "Casey"}'

# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "producer@acme.com", "password": "Password1!"}'
# → { "access_token": "eyJ...", "refresh_token": "..." }

# Create a movie
curl -X POST http://localhost:8080/api/movies \
  -H "Authorization: Bearer eyJ..." \
  -H "Content-Type: application/json" \
  -d '{"title": "Cyber City", "director": "Jane Doe", "release_year": 2026}'

# Listen for license events (SSE)
curl -N http://localhost:8080/api/licenses/events
```

Full API documentation is available at http://localhost:8080/docs after starting the backend.

## Testing

The backend has **~490 tests** across unit, integration, and API levels:

```bash
cd backend

# Run all tests (requires Docker for Testcontainers)
cargo test

# Run a specific test module
cargo test --test license_tests
cargo test --test license_api_tests
```

Tests use [Testcontainers](https://testcontainers.com/) to spin up real PostgreSQL instances — no mocks, no SQLite substitution. Each test file gets an isolated database with migrations applied.

| Test file | Count | What it covers |
|-----------|-------|----------------|
| `license_tests` | 46 | Full negotiation workflow: create, submit, counter-offer, accept, reject, cancel, authorization checks |
| `license_api_tests` | 28 | HTTP-level license endpoint tests (status codes, response shapes, SSE streaming) |
| `song_tests` | 35 | CRUD, artist/label association, search, pagination |
| `movie_tests` | 31 | CRUD, team membership, role-based access |
| `label_tests` | 26 | CRUD, member management, role transitions |
| `auth_tests` | 25 | Registration, login, token refresh, session management |
| `track_tests` | 22 | Track placement, time range validation, song-scene linking |
| `scene_tests` | 19 | CRUD, timecode validation, movie association |
| `role_tests` | 19 | RBAC, scope resolution, role assignment |
| `user_tests` | 15 | Profile management, status transitions |
| `auth_api_tests` | 10 | HTTP-level auth endpoint tests |
| Service unit tests | ~215 | Inline `#[tokio::test]` tests in each service module |

## Tech Decisions and Tradeoffs

### Why REST instead of GraphQL?

The domain has a clear resource hierarchy (movies → scenes → tracks → licenses → offers) that maps naturally to REST endpoints. The negotiation workflow is action-oriented (submit, counter-offer, accept, reject) — these are better expressed as dedicated POST endpoints than generic mutations.

GraphQL would add value if clients needed flexible field selection across deeply nested data, but the current UI makes predictable queries that REST serves well. The SSE stream handles the real-time requirement that GraphQL subscriptions would otherwise address.

### Why SSE instead of WebSockets?

License negotiation events are server-to-client: "someone submitted an offer," "the other side accepted." The client doesn't need to push data through the same channel — it uses regular POST requests for actions. SSE is simpler (no handshake upgrade, no ping/pong, automatic reconnection built into `EventSource`), works through HTTP/2 multiplexing, and needs no additional infrastructure.

WebSockets would be warranted if we needed bidirectional streaming (e.g., real-time collaborative editing of license terms), but the current workflow is request/response for actions + server push for notifications.

### Error handling

The API returns structured error responses with machine-readable codes, human-readable messages, error categories, and optional detail maps:

```json
{
  "code": "CONFLICT",
  "message": "A license request already exists for this track",
  "error_type": "Conflict",
  "details": { "track_id": "abc-123" }
}
```

Internal errors are redacted from clients (generic message, details stripped) while the full error is logged server-side.

## Project Structure

```
.
├── backend/
│   ├── src/
│   │   ├── iam/              # Auth, users, roles, sessions
│   │   ├── movie/            # Movies, movie members
│   │   ├── scene/            # Scenes
│   │   ├── song/             # Songs
│   │   ├── track/            # Track placements
│   │   ├── label/            # Labels, label members
│   │   ├── license/          # License requests, offers, SSE
│   │   ├── kernel/           # Shared types
│   │   ├── error/            # Error types and responses
│   │   ├── openapi.rs        # OpenAPI spec generation
│   │   └── main.rs           # Server bootstrap
│   ├── migrations/           # SQL migrations
│   ├── tests/                # Integration + API tests
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── api/              # API client (real + mock backends)
│   │   ├── components/       # Shared UI components + shadcn/ui primitives
│   │   ├── lib/              # Hooks, utilities, persona system
│   │   ├── pages/
│   │   │   ├── studio/       # Movie team workspace
│   │   │   └── rights/       # Rights holder workspace
│   │   └── App.tsx           # Router
│   ├── Dockerfile
│   └── nginx.conf
├── compose.yml
├── .env.example
└── README.md
```

## Development (without Docker)

If you prefer running services directly:

```bash
# Terminal 1: PostgreSQL
docker run -d --name music-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=music_licensing -p 5432:5432 postgres:16-alpine

# Terminal 2: Backend
cd backend
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/music_licensing
cargo run

# Terminal 3: Frontend (real API mode)
cd frontend
npm install
npm run dev
# Open http://localhost:5173, switch to "Real" API mode in the UI

# Frontend (mock mode, no backend needed)
cd frontend
npm install
npm run dev
# Open http://localhost:5173, keep "Mock" API mode (default)
```

## Deployment (AWS)

Terraform for a production deployment (ECS Fargate backend behind an ALB, S3 + CloudFront for the frontend, RDS Postgres) lives in [`infra/terraform`](infra/terraform). See [`infra/README.md`](infra/README.md) for architecture notes and deploy steps.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/music_licensing` | PostgreSQL connection string |
| `JWT_SECRET` | dev default (insecure) | Secret key for JWT signing (min 32 chars) |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `CORS_ORIGIN` | `*` | Allowed CORS origin (set to frontend URL in production) |
| `BIND_ADDR` | `0.0.0.0:8080` | Backend listen address |
| `DB_MAX_CONNECTIONS` | `10` | PostgreSQL connection pool size |
| `ACCESS_TOKEN_TTL_SECS` | `900` | JWT access token lifetime |
| `REFRESH_TOKEN_TTL_SECS` | `604800` | JWT refresh token lifetime |
