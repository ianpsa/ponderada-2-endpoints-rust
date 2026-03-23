# Telemetry Collector

Backend pra coleta assíncrona de dados de sensores industriais usando Rust, RabbitMQ e SQLite.

## Decisões Técnicas

- **Rust** ao invés de Go pq ja tenho familiaridade e o desempenho é comparável (ou melhor). O ecossistema async com tokio resolve bem o problema de concorrência.
- **axum** como framework HTTP, async, leve, type-safe e faz parte do ecossistema tokio.
- **lapin** pra integração com RabbitMQ, é a crate mais usada pra conectar com Rabbit em Rust.
- **SQLite** ao invés de PostgreSQL pra simplificar a infra. Sem precisar de mais um container rodando e pra esse volume de dados funciona bem. O r2d2 cuida do connection pooling.
- **Docker Compose** orquestrando o RabbitMQ e o app. O SQLite fica num volume persistente.

## Arquitetura

```
POST /telemetry -> RabbitMQ (fila) -> Consumer (background) -> SQLite
```

O endpoint recebe o payload do sensor, valida e joga na fila. O consumer roda como task em background no mesmo binário, consome as mensagens e persiste no banco. Se o consumer cair ele reconecta automaticamente a cada 5s.

### Tipos de sensores suportados

| Sensor | Natureza | Valor |
|--------|----------|-------|
| temperature | analog | float (°C) |
| humidity | analog | float (%) |
| presence | discrete | bool |
| vibration | analog | float |
| luminosity | analog | float (lux) |
| reservoir_level | analog | float (%) |

## Como Rodar

```bash
docker compose up --build
```

O app sobe na porta 8080. RabbitMQ management na 15672 (guest/guest).

### Teste manual

```bash
curl -X POST http://localhost:8080/telemetry \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "dev-001",
    "timestamp": "2026-03-22T10:00:00Z",
    "sensor_type": "temperature",
    "reading_nature": "analog",
    "value": 23.5
  }'
```

Resposta esperada: `202 Accepted` com `{"status": "enqueued"}`.

### Teste de Carga

```bash
k6 run k6/load_test.js
```

O script simula 5 dispositivos enviando leituras de diferentes sensores com ramp-up gradual até 100 VUs simultâneos.

## Análise dos Resultados do k6

Cenário: ramp-up de 10 -> 50 -> 100 VUs ao longo de 2min30s.

| Métrica | Valor |
|---------|-------|
| total de requests | 55.271 |
| throughput médio | ~368 req/s |
| latência média | 1.64ms |
| latência p50 | 1.47ms |
| latência p95 | 3.09ms |
| latência máxima | 94.69ms |
| taxa de erro | 0% |
| checks passando | 100% (110.542/110.542) |

Aguentou bem os 100 VUs sem perder nenhuma mensagem. A fila do RabbitMQ ficou com 0 mensagens pendentes (ready=0) após o teste, confirmando que o consumer processou tudo com sucesso. A latência p95 de 3.09ms mostra que o pipeline está bem otimizado.

## Estrutura do Projeto

```
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── k6/
│   └── load_test.js
└── src/
    ├── main.rs            # entrypoint, wiring
    ├── config.rs          # env vars
    ├── db.rs              # SQLite pool, migrations, insert
    ├── error.rs           # tipos de erro do axum
    ├── models.rs          # payload, enums, conversão
    ├── handlers/
    │   ├── mod.rs
    │   └── telemetry.rs   # POST /telemetry
    └── queue/
        ├── mod.rs
        ├── publisher.rs   # publica na fila
        └── consumer.rs    # consome e persiste
```
