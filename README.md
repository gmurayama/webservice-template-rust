# Webservice Template in Rust

This project serves as a template for building web services in Rust using [actix-web](https://actix.rs). It provides a basic structure and setup to kickstart your web service development.

It takes a lot of inspiration on [Zero To Production in Rust](https://github.com/LukeMathWalker/zero-to-production), a must read book by Luca Palmieri -- if you haven't already, I highly recommend checking it out!

## But Why?

A huge portion of the codebase is dedicated to not only organizing code in cohesive groups, but also to provide observability, handle configuration, easier the development with "helper" functions -- which can be deceiftul longer to do than one can expect. 

Hopefully this template can provide you with everything to launch an app as close to production-ready as possible by minimizing the initial overhead of creating a project (even though you may not require all the features and the additional complexity of the template).

As for the decisions made, most probably you will disagree with some of them, and that's fine! It really shouldn't be difficult to change them to fit better your needs (see [Architecture decisions](#architecture-decisions)).

## Overview

### Features

- Handle [configuration](https://github.com/mehcode/config-rs) on the application using environment variables.
- By default, it has a middleware that timeout a request that takes too long
- Emit traces using the [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust) framework any OTel Collector (such as Jaeger).
- Use [Prometheus](https://github.com/prometheus/client_rust) to send metrics following the Open Metrics specification.

### Structure

```
webservice-template-rust/
├─ application/           # business layer
├─ gateways/              # entry points (all the binaries)
│  ├─ api/
│  │  ├─ routes/          # all functions that are going to be registered on server.rs
│  │  ├─ main.rs
│  │  ├─ settings.rs      # all the settings used in the API (telemetry, metrics, host:port, etc.)
│  ├─ .../
├─ infrastructure/        # adapters
├─ docker-compose.yaml    # prometheus, grafana and jaeger services (development purposes)
├─ prometheus.yaml        # basic config for prometheus
```

## Getting Started

1. Customize the project for your needs:
    - Add new routes following the example on [routes/](gateways/api/src/routes/reply.rs)
    - Configure environment variables or settings in [settings.rs](gateways/api/src/settings.rs) as per your requirements.

2. Run the project:

```bash
$ docker-compose up -d
$ cargo run
```

Access your web service at http://localhost:7000 (by default).

## Architecture decisions

- Each layer is a different crate: `application` and `infrastructure` are libraries and `api` is a binary. It makes easier to create other binaries (for example, a CLI) that can use the same crates that `api` is using instead of generating a single binary that can be both.
- The naming convention on the crates does not follow Rust standard -- usually there is a prefix with the project name and the binaries are all in the root folder. It is mostly a personal preference, but changing the name to be more idiomatic should be fairly simple.
- The `timeout_middleware` is scoped on `/v1` so it cancels **only** the request. If it was at the same level as the other middleware, all code that should run after the request is cancelled and **will not** run as intended (for example, the metrics middleware relies on emitting metrics after the request is finished).
  - All code inside the route **must** be cancel safe if the `timeout_middleware` is enabled. The problem is better described in the talk "[Async Rust: the good, the bad, and the ugly](https://youtu.be/1zOd52_tUWg?si=tQ6ndEi0XuepuE76&t=1962)" by Steven Klabnik. Consider using [cancel-safe-futures](https://docs.rs/cancel-safe-futures/latest/cancel_safe_futures/) crate.
- The code structure follows the Hexagonal Architecture as I find to be quite easy to maintain and easier the burden of deciding where each piece of code should be. Basically `infrastructure` is where all outbound ports lives, `gateways` is where all the main entry points resides (inbound ports) and `application` is the "bussiness" layer.
- [Dockerfile](Dockerfile) leverages multi architecture build with a caching strategy to speed up build times, **however** they can make the build take even longer if the build host is always chaging (therefore, it never uses the cache layer) or you only deploy to x86-64 architectures. If that's the case, refer to this [simplified Dockerfile](https://github.com/gmurayama/webservice-template-rust/blob/7180e56dcec21e324991ce1cde83192b7cb32ef1/Dockerfile) that builds only to one target arch
  - The choice of using alpine is to avoid the trouble of downloading zig from the source. Since the Dockerfile is meant to work on either arm64 or amd64, relying on the package manager saves the trouble of choosing the file based on the build platform.
  - The Dockerfile was taken from this [article](https://medium.com/@vladkens/fast-multi-arch-docker-build-for-rust-projects-a7db42f3adde), but I found out the hard way that building for `musl` can be quite the challenge. It was not worth it, so I changed to `gnu` and used `debian:bullseye-slim` for the final image.

## License

This project is licensed under the [MIT License](LICENSE).
