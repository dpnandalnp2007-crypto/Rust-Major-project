# Rust-Major-projectt
# 🛡️ Identity & Access Management (IAM) Platform

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16.0-336791.svg?logo=postgresql)
![Testing](https://img.shields.io/badge/Tests-Passing-brightgreen.svg)
![Architecture](https://img.shields.io/badge/Architecture-Clean-success.svg)

> A production-grade, multi-tenant Identity and Access Management (IAM) backend engineered in Rust. Designed to provide secure authentication, granular Role-Based Access Control (RBAC), and centralized tenant routing for downstream enterprise services.

---

## ✨ Core Features

- **Multi-Tenant Architecture:** Seamlessly manage users across isolated organizations. A single user can hold different roles in different organizations.
- **Granular RBAC:** Dynamic Permission evaluation (`role:create`, `user:delete`) mapped through custom organizational roles.
- **Robust Security:** JWT-based stateless authentication, cryptographically hashed passwords (Argon2), and scoped machine-to-machine API keys.
- **Audit Traceability:** Immutable audit logging for all critical state mutations.
- **Pluggable Persistence:** Utilizes the Repository Pattern to hot-swap between a live PostgreSQL cluster and an in-memory datastore for blazing-fast integration testing.

---

## 🛠️ Technology Stack

| Category | Technology | Purpose |
| :--- | :--- | :--- |
| **Core Language** | `Rust` | Memory safety, zero-cost abstractions, thread safety. |
| **Web Framework** | `Axum` / `Tokio` | High-performance, asynchronous HTTP routing. |
| **Database** | `PostgreSQL` | Relational data integrity and JSONB support. |
| **Persistence** | `SQLx` | Compile-time validated SQL queries and migrations. |
| **Security** | `jsonwebtoken` / `argon2` | Secure session management and credential hashing. |

---

## 📂 Project Structure

The codebase strictly adheres to a decoupled, layered architecture to guarantee separation of concerns.

```text
.
├── migrations/             # SQLx database schema & seed data
├── src/
│   ├── config.rs           # Environment & state configuration
│   ├── main.rs             # Application entry point & TCP binding
│   ├── routes.rs           # Axum router & middleware orchestration
│   ├── handlers/           # HTTP transport layer (Controllers)
│   ├── middleware/         # Strongly-typed Auth Extractors & RBAC guards
│   ├── services/           # Core business logic & domain rules
│   ├── repositories/       # Data access abstraction (Postgres & Memory)
│   ├── models/             # Database entity representations
│   └── dto/                # Request validation & Response payloads
└── tests/
    ├── unit_tests.rs       # Isolated cryptography & pure function tests
    └── integration_tests.rs# End-to-end HTTP request flows (Memory DB)
```

## 6. Architecture Diagram
The platform follows a layered, decoupled architecture to ensure clear separation of concerns and maintainability.

```mermaid
flowchart TD
    Client([Client / Frontend]) --> Router[Axum Router]
    
    subgraph Middleware
        Router --> Auth[AuthUser Extractor]
        Router --> RBAC[Require Permission]
    end
    
    subgraph Application Layer
        Auth --> Handlers[Route Handlers]
        RBAC --> Handlers
        Handlers --> Services[Business Logic Services]
    end
    
    subgraph Data Layer
        Services --> Repo[AppRepository Trait]
        Repo --> Postgres[PostgresRepository]
        Repo --> Memory[MemoryRepository for Tests]
        Postgres --> DB[(PostgreSQL Database)]
    end
