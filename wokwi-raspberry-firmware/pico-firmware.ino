#include <WiFi.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>
#include <NTPClient.h>
#include <WiFiUdp.h>
#include <time.h>

// configs wokwi e host da fila
const char* wifi_nome = "Wokwi-GUEST";
const char* wifi_senha = ""; 
const char* url_servidor = "<HOST>/telemetry"; // host do servidor, no meu caso usei localtunnel

int pino_pir = 15;     
int pino_ldr = 26;     
unsigned long tempo_anterior = 0;

// 0: conectar, 1: ler e 2: enviar
int estado = 0; 

String tipo_post = "";
String natureza_post = ""; 
float valor_post = 0.0;

WiFiUDP ntpUDP;
NTPClient timeClient(ntpUDP, "pool.ntp.org", 0, 0); 

String pegar_tempo() {
  time_t epochTime = timeClient.getEpochTime();
  struct tm *ptm = gmtime(&epochTime);
  char buffer[25];
  sprintf(buffer, "%04d-%02d-%02dT%02d:%02d:%02dZ", 
          ptm->tm_year + 1900, ptm->tm_mon + 1, ptm->tm_mday, 
          ptm->tm_hour, ptm->tm_min, ptm->tm_sec);
  return String(buffer);
}

void setup() {
  Serial.begin(115200);
  pinMode(pino_pir, INPUT);
  analogReadResolution(12);
  Serial.println("Sistema iniciado...");
}

void loop() {
  switch (estado) {
    
    case 0: // CONECTA NO WIFI
      Serial.print("Conectando no WiFi...");
      WiFi.begin(wifi_nome, wifi_senha);
      while (WiFi.status() != WL_CONNECTED) {
        delay(500);
        Serial.print(".");
      }
      Serial.println("\nConectado!");
      timeClient.begin();
      
      // sincronizar hora para o timestamp
      Serial.println("Sincronizando hora...");
      while(!timeClient.update()){
        timeClient.forceUpdate();
        delay(500);
        Serial.print("t");
      }
      Serial.println("\nHora sincronizada: " + timeClient.getFormattedTime());
      
      estado = 1; 
      break;

    case 1: // LER SENSOR
      timeClient.update();
      
      if (digitalRead(pino_pir) == HIGH) {
        tipo_post = "presence";
        natureza_post = "discrete"; 
        valor_post = 1.0;
        estado = 2; 
        delay(500);
      } 
      else if (millis() - tempo_anterior > 10000) {
        tempo_anterior = millis();
        int leitura = analogRead(pino_ldr);
        valor_post = (leitura / 4095.0) * 100.0;
        tipo_post = "luminosity";
        natureza_post = "analog"; 
        estado = 2; 
      }
      break;

    case 2: // ENVIAR
      Serial.print("Sensor: "); Serial.println(tipo_post);
      
      WiFiClientSecure client; 
      client.setInsecure(); // Necessário para o Wokwi aceitar HTTPS
      
      HTTPClient http;
      http.setTimeout(30000);
      http.setReuse(false); 
      
      if (http.begin(client, url_servidor)) {
        http.addHeader("Content-Type", "application/json");
        http.addHeader("bypass-tunnel-reminder", "true");
        http.addHeader("Connection", "close");

        JsonDocument doc; 
        doc["device_id"] = "PICO-01";
        doc["timestamp"] = pegar_tempo();
        doc["sensor_type"] = tipo_post;
        doc["reading_nature"] = natureza_post;
        
        if (tipo_post == "presence") {
          doc["value"] = true;
        } else {
          doc["value"] = valor_post;
        }

        String corpo;
        serializeJson(doc, corpo);
        Serial.println("Payload: " + corpo);
        
        int code = http.POST(corpo);
        
        if (code > 0) {
          Serial.print("Sucesso! Codigo: ");
          Serial.println(code);
        } else {
          Serial.print("Erro no POST (code ");
          Serial.print(code);
          Serial.print("): ");
          Serial.println(http.errorToString(code).c_str());
        }
        http.end();
      } else {
        Serial.println("Nao foi possivel conectar ao servidor.");
      }
      
      estado = 1; 
      break;
  }
}
