# Webservice Template in Rust

This project serves as a template for building web services in Rust using the actix-web framework. It provides a basic structure and setup to kickstart your web service development.

## Features

- actix-web: utilizes Actix-web, a powerful and pragmatic framework for building web applications and web services in Rust.
- OpenTelemetry Integration: includes OpenTelemetry support for distributed tracing and observability, allowing you to monitor and trace requests across your service.
- Configurable: easily configurable with environment variables or configuration files to adapt to different deployment environments.
- Logging: integrated logging setup for better debugging and monitoring.

## Getting Started

To get started with your own project using this template, follow these steps:

1. Customize the project for your needs:
  - Add new routes following the example on [routes/](gateways/api/src/routes/reply.rs)
  - Configure environment variables or settings in [settings.rs](gateways/api/src/settings.rs) as per your requirements.

2. Run the project:

```bash
docker-compose up -d
cargo run
```

Access your web service at http://localhost:7000 (by default).

## License

This project is licensed under the [MIT License](LICENSE).
