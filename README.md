# Rust Realtime Metrics Exporter 🦀⚡

Exportador de telemetría y métricas de infraestructura de ultra-alta velocidad y sub-milisegundo construido en **Rust**, respaldado por el runtime asíncrono **Tokio**, el framework web **Axum** y especificaciones **Prometheus / OpenTelemetry**.

[![Rust](https://img.shields.io/badge/Language-Rust%201.78+-orange.svg)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Async-Tokio-blue.svg)](https://tokio.rs/)
[![Prometheus](https://img.shields.io/badge/Metrics-Prometheus-red.svg)](https://prometheus.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-purple.svg)](https://opensource.org/licenses/MIT)

---

## 🎯 El Problema de Ingeniería

En arquitecturas Kubernetes de alta densidad, plataformas en la nube y microservicios críticos, los recolectores de métricas tradicionales basados en lenguajes interpretados o con Garbage Collection (GC) generan:

1. **Picos de Latencia por GC Pauses:** Interrupciones imprevistas durante la recolección de estadísticas que distorsionan los SLAs de observabilidad.
2. **Alto Consumo de Memoria:** Recolectores pesados que consumen recursos valiosos de CPU/RAM destinados a las aplicaciones core.
3. **Bloqueos por Lock Contention:** Cuellos de botella al actualizar contadores globales entre múltiples hilos concurrentes.

---

## 💡 La Solución

**Rust Realtime Metrics Exporter** garantiza recolección sin pausas de Garbage Collection, seguridad de memoria en tiempo de compilación y contadores atómicos sin bloqueos (*lock-free*).

```
 ┌────────────────────────────────────────────────────────┐
 │                   System Telemetry                     │
 │ (CPU Usage, Memory RSS, Disk I/O, Active Network Sockets)│
 └──────────────────────────┬─────────────────────────────┘
                            │ (Sub-millisecond Non-blocking)
                            ▼
 ┌────────────────────────────────────────────────────────┐
 │               MetricsCollector (Rust Tokio)             │
 │    - Atomic U64 / F64 Counters (std::sync::atomic)     │
 │    - Lock-Free Concurrent Registry                     │
 └──────────────────────────┬─────────────────────────────┘
                            │
                            ▼
 ┌────────────────────────────────────────────────────────┐
 │                Axum HTTP Web Endpoint                  │
 │           GET /metrics (Prometheus Format)             │
 └────────────────────────────────────────────────────────┘
```

### ✨ Características Clave
- **Cero Overhead de Garbage Collection:** Seguridad de memoria garantizada por el modelo de *ownership* y *borrow checker* de Rust.
- **Estructura Lock-Free Atómica:** Recolección concurrente multitarea utilizando tipos atómicos (`AtomicU64`, `AtomicF64`).
- **Formato Estándar Prometheus:** Exposición en `/metrics` compatible de forma nativa con Grafana, Datadog y Prometheus Scrapers.
- **Alert Trigger Engine:** Monitoreo en background con disparadores de webhooks configurables ante picos anómalos de recursos.

---

## 🛠️ Estructura del Código Core

- `src/metrics/collector.rs`: Módulo principal de recolección asíncrona, actualización atómica de métricas y formateo Prometheus.

---

## 💻 Ejemplo de Uso

```rust
use rust_realtime_metrics_exporter::metrics::collector::MetricsCollector;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let collector = Arc::new(MetricsCollector::new());
    
    // Registrar métricas de sistema
    collector.record_cpu_usage(42.5);
    collector.increment_request_count("GET", "/api/v1/health");

    // Formatear salidas para Prometheus
    let prometheus_payload = collector.export_prometheus_format();
    println!("{}", prometheus_payload);
}
```

---

*Desarrollado por Esteban Maximiliano Aulestia Andrade - Systems & Software Engineer.*