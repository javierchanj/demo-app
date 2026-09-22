# Loco.rs → Docker

A beginner walkthrough. You will build a small Rust REST API with Loco, package it with Docker

**How to use this guide:** do the phases in order and don't skip the checkpoints because it halp keep track of progress. Each
checkpoint tells you what you should see before moving on. Most beginner pain comes from
debugging five things at once. The checkpoints make sure you only ever debug one.

**Time:** about 1-2 hours the first time. Rust Docker builds can be slow, so expect waiting.

---

## First Install the tools

You need these on your machine:

| Tool | Install | Check it works |
|---|---|---|
| Rust | https://rustup.rs | `cargo --version` |
| Docker Desktop | https://docs.docker.com/get-docker/ | `docker run hello-world` |

---

## Phase 1 - Create the Loco app

### 1.1 Install the Loco generator

```
cargo install loco
cargo install sea-orm-cli
```

This takes a few minutes.

### 1.2 Generate the app

```
loco new --name demo_app --db postgres --bg async
cd demo_app
```

`loco new` then asks two interactive questions. Answer them like this:

```
✔ ❯ What would you like to build? · Saas App with server side rendering
✔ ❯ Embed static assets into the binary? · no
```

This guide picks **Saas App with server side rendering**, so the frontend is server-rendered
by the app itself (no separate JS build step). Answering **no** to embedding assets keeps
the static files on disk instead of compiling them into the binary, which is simpler while
you're iterating locally.

What the flags mean:

- `--db postgres` uses a Postgres server.
- `--bg async` runs background jobs inside the app process.


### 1.2b Start a local Postgres server

Docker needs its engine running before any `docker run` command will work. Start **Docker
Desktop** first (Start Menu → Docker Desktop, or launch it from the taskbar), or launch it
from a terminal:
Reference: https://docs.docker.com/reference/cli/docker/desktop/

Use preferred command depending on your OS
```
# PowerShell / Git Bash on Windows
"C:\Program Files\Docker\Docker\Docker Desktop.exe" 

# macOS
open -a Docker

# linux
systemctl --user start docker-desktop

# Windows, macOS
docker desktop start
```

Wait for the whale icon in the system tray to stop animating and show "Engine running."
Confirm with:

```
docker info
```

The generated `config/development.yaml` expects Postgres reachable at
`postgres://loco:loco@localhost:5432/demo_app_development`. The easiest way to get that
locally is Docker:

```
  docker run -d --name demo_app_pg -p 5432:5432 -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco -e POSTGRES_DB=demo_app_development postgres:16
```

Open `config/development.yaml` and confirm the `database.uri` line matches this (user,
password, host, port, database name). Edit either the file or the `docker run` command so
they agree.

✅ **Checkpoint:** `docker ps` shows `demo_app_pg` as `Up`.

### 1.3 Run it

```
cargo loco start
```

The first run compiles for a while. When you see `listening on port 5150`, open a
**second terminal** and run:

```
curl localhost:5150/_ping
# {"ok":true}
```

✅ **Checkpoint:** you get `{"ok":true}`. Stop the server with `Ctrl+C`.

Remember `/_ping`. It's the health check path AWS will use later.

### 1.4 Add a CRUD resource

We will create a simple table to store post with two fields: title and published. 

```
cargo loco generate scaffold posts title:string! published:bool --no-auth
cargo loco db migrate
```

- `title:string!` makes the title required (the `!` means "not null").
- `published:bool` is a field added to indicate if the post is still in draft or has been published and you can omit it when creating a post.



To list all the routes the scaffold just added so you know what to test. 

```
cargo loco routes
```

This doesn't need the server running, it's a standalone CLI command that reads your router config directly. It literally prints every route the app exposes (both the JSON API under `/api/posts` and since
this is an SSR app, the HTML view routes under `/posts`). Use it any time you're not sure what path to hit.

Now you want to test the routes you have seen. Start the app again and try it. Remember to stop the existing running server instance with `Ctrl+C` before starting the app again, else you experience an os lock error.:

```
cargo loco start
```

After app is started, in the second terminal test with:

```
curl -X POST localhost:5150/api/posts \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Post With Loco", "published": True}'
```
```
curl localhost:5150/api/posts
```

✅ **Checkpoint:** the second command returns a JSON list containing your post.

### 1.5 Run the tests

`cargo test` uses a separate database from the one you started in 1.2b - `config/test.yaml`
expects `demo_app_test` on the same Postgres container. Create it first (one-time, using the
`demo_app_pg` container from 1.2b):

```
docker exec -it demo_app_pg psql -U loco -d demo_app_development -c "CREATE DATABASE demo_app_test;"
```

Then run the tests:

```
cargo test
```

✅ **Checkpoint:** all tests pass. The pipeline will run exactly this command, so it has to
pass locally first.

### 1.6 Put it on GitHub

Create an **empty** repo on GitHub named `demo-app` (no README, no .gitignore). Then:

```
git add .
git commit -m "Initial Loco app with posts scaffold"
git push
```

---

## Phase 2 - Dockerise it and run it locally

### 2.1 Generate the Dockerfile

Loco can write this for you:

```
cargo loco generate deployment docker
```

This creates `Dockerfile` and `.dockerignore`. Open the Dockerfile and read it. You should
recognise two stages:

1. a **builder** stage that runs `cargo build --release`
2. a small **runtime** stage (`debian:bookworm-slim`) that copies in only the compiled
   binary and the `config/` folder

Find the binary name in the Dockerfile. It's your app name with `-cli` on the end, e.g.
`demo_app-cli`.

**Make the container run the server by default.** As generated, the Dockerfile's
`ENTRYPOINT` is just the binary with no subcommand:

```
ENTRYPOINT ["/usr/app/demo_app-cli"]
```

Run as-is, `docker run demo-app` doesn't start anything - the CLI has no default subcommand,
so it just prints its `--help` output and exits. Bake `start` into the entrypoint so the
container's whole job is running the server:

```
ENTRYPOINT ["/usr/app/demo_app-cli", "start"]
```

This means `docker run demo-app` always starts the server, with no way to run a different
subcommand (like `routes` or `db migrate`) against this image - for a production image whose
only job is serving traffic, that's the right tradeoff.

**Before building, check the Rust version matches, still in the same file at the very top.** The builder stage's first line pins an
exact Rust version, e.g.:

```
FROM rust:1.92.0-slim AS builder
```

Compare that to what's installed on your machine:

```
rustc --version
```

If your local `rustc` is **newer** than the version pinned in the Dockerfile, your
`Cargo.lock` may contain dependency versions that require a newer compiler than the one
inside the container - the build will fail partway through `cargo build --release` with an
error like `requires rustc X but this is Y`. Edit the Dockerfile's `FROM` line so it matches the exact
version on your local `rustc`, so your local build and the container
build use the same compiler and behave consistently:

```
FROM rust:1.98.0-slim AS builder
```

**Also pin the builder stage to the same Debian release as the runtime stage.** The
Dockerfile's two stages use different base images - `rust:...-slim` for the builder and
`debian:bookworm-slim` for the runtime - and if you only match the Rust *version* (above)
without also matching the Debian *release*, `rust:X-slim` can silently resolve to a newer
Debian release than `bookworm` (e.g. `trixie`), which ships a newer glibc. The binary then
gets linked against a newer glibc than the runtime stage has, and the container fails
immediately with:

```
/usr/app/demo_app-cli: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by /usr/app/demo_app-cli)
```

Fix it by adding `-bookworm` explicitly to the builder's tag, so both stages share the same
glibc:

```
FROM rust:1.98.0-slim-bookworm AS builder
```

### 2.2 Now we create a local production env file

In production, Loco reads `config/production.yaml`, and that file expects its secrets to
come from **environment variables**. Because the app now uses Postgres, the container also
needs a Postgres server it can reach - the container running the app is not the database.

Start a second container for the production database, on a different host port so it
doesn't clash with the dev one from 1.2b:

```
docker run -d --name demo_app_pg_prod -p 5433:5432 \
  -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco -e POSTGRES_DB=demo_app_production \
  postgres:16
```

Create `.env.production` in the project root. Use `host.docker.internal` so the app
container can reach the Postgres container running on your machine:

```
cat > .env.production <<'EOF'
LOCO_ENV=production
DATABASE_URL=postgres://loco:loco@host.docker.internal:5433/demo_app_production
DB_AUTO_MIGRATE=true
JWT_SECRET=local-test-secret-change-me-0123456789abcdef
HOST=http://localhost:5150
EOF

echo ".env.production" >> .gitignore
```

Why each variable matters:

- `LOCO_ENV=production` is essential. Without it, Loco quietly uses the development
  config.
- `DATABASE_URL` points at the Postgres container. `host.docker.internal` lets a container
  reach a service running on your host machine; on Linux you may instead need
  `--add-host=host.docker.internal:host-gateway` on the `docker run` command below, or use
  the host's real IP.
- `DB_AUTO_MIGRATE=true` creates the tables when the app boots.
- `JWT_SECRET` is required because the app includes Loco's login system, even though our
  post routes don't use it.
- `HOST` is the app's public URL, used in email links. A placeholder is fine for now.

**Disable the mailer for this tutorial.** 

Find and replace this section in your `config/production.yaml`. As an alternative, a sample `production.yaml` is provided for your use. 

```yaml
mailer:
  smtp:
    enable: false
    host: "<%= get_env(name="MAILER_HOST", default="") %>"
    port: <%= get_env(name="MAILER_PORT", default="587") %>
    secure: true
    auth:
      user: "<%= get_env(name="MAILER_USER", default="") %>"
      password: "<%= get_env(name="MAILER_PASSWORD", default="") %>"
```

### 2.3 Build and run the container

```
docker build -t demo-app .
docker run --rm -p 5150:5150 --env-file .env.production demo-app
```

> **Apple Silicon Mac?** Use `docker build --platform linux/amd64 -t demo-app .` so the
> image matches the Linux servers AWS runs.

The first build takes 5–15 minutes. In a second terminal:

```
curl localhost:5150/_ping
```
```
curl -X POST localhost:5150/api/posts -H "Content-Type: application/json" -d '{"title":"from docker","published":true}'
```
```
curl localhost:5150/api/posts
```

✅ **Checkpoint:** all three commands work against the **container**.


Commit the Dockerfile:

```
git add Dockerfile .dockerignore .gitignore
git commit -m "Add Dockerfile"
git push
```

---
