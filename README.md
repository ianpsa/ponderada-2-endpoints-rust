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

## Atividade Ponderada 2: Integração com Raspberry Pi Pico W

Esta fase do projeto integra um dispositivo embarcado (Raspberry Pi Pico W) para coleta de dados de sensores físicos e envio para o backend assíncrono.

### Especificações Técnicas
- **Framework:** Arduino Framework (C++).
- **Plataforma de Simulação:** Wokwi.
- **Protocolo de Comunicação:** HTTP/HTTPS com JSON.
- **Gerenciamento de Estado:** Máquina de Estados Finita (FSM) com 3 estados principais (Conexão, Leitura, Envio).

### Sensores Integrados
| Sensor | Tipo | Pino GPIO | Range/Escala | Descrição |
|--------|------|-----------|--------------|-----------|
| **PIR (HC-SR501)** | Digital | 15 | 0 ou 1 | Detecta movimento/presença. |
| **LDR (Fotoresistor)**| Analógico| 26 (ADC0) | 0 a 100% | Mede a luminosidade ambiente. |

### Configuração e Gravação
1. **SSID/Senha:** No arquivo `pico-firmware.ino`, altere as variáveis `wifi_nome` e `wifi_senha`. No Wokwi, use `Wokwi-GUEST` e senha vazia.
2. **Endpoint:** Altere `url_servidor` para o seu link do Localtunnel (ex: `https://<url_servidor>/telemetry`).
3. **Compilação:** Utilize a IDE do Arduino com a placa "Raspberry Pi Pico W" instalada ou o simulador Wokwi carregando o arquivo `.ino` e o `diagram.json`.

### Diagrama de Conexão (Wokwi)
- **PIR:** VCC -> 5V, GND -> GND, OUT -> GP15.
- **LDR:** Conectado em divisor de tensão no pino GP26 (ADC0).

### Evidências de Funcionamento
- **Logs Seriais:** O firmware loga a sincronização de tempo (NTP), o estado da conexão WiFi e o status do envio (HTTP 202).
- **Backend:** O servidor Axum loga `Recebido:` e a persistência no banco SQLite pode ser verificada via comando `sqlite3`.

### Como Rodar os Testes de Código (Rust)
Para garantir que a lógica de conversão e os modelos de dados estão corretos, adicionei testes unitários no backend. Para rodá-los:
```bash
cargo test
```
*Estes testes validam se o JSON recebido do Wokwi é transformado corretamente em uma leitura válida para o banco.*

### Como Ver os Dados no Banco (SQLite)
Como o banco de dados roda dentro de um volume do Docker para persistência, você deve usar o seguinte comando para visualizar as leituras em tempo real:
```bash
sudo docker run --rm -v ponderada-2-endpoints_sqlite_data:/data alpine sh -c "apk add --no-cache sqlite && sqlite3 -column -header /data/telemetry.db 'SELECT * FROM telemetry ORDER BY id DESC LIMIT 10;'"
```
*Este comando baixa uma imagem temporária do SQLite e consulta o arquivo `/data/telemetry.db` montado no volume.*

---

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
