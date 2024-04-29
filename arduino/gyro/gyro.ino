/*
  Este script que o arduino deve rodar para funcionar com esta a biblioteca gyro_lib. Ele lê
  a rotação de um MPU6050 e transmite os dois eixos via serial e, se disponivel, UDP. A 
  biblioteca se responsabiliza por ler e tratar os valores. Para que a transmissão por UDP
  funcione, a placa deve ser um NodeMCU e o computador deve se conectar com o wifi 
  "rede_gyro".


  Baseado em: https://wiki.dfrobot.com/How_to_Use_a_Three-Axis_Accelerometer_for_Tilt_Sensing

  Conexões:

  MPU6050  NodeMCU      Description
  ======= ==========    ====================================================
  VCC     VU (3.3V)     Power
  GND     G             Ground
  SCL     D1 (GPIO05)   I2C clock
  SDA     D2 (GPIO04)   I2C data
  XDA     not connected
  XCL     not connected
  AD0     not connected
  INT     D8 (GPIO15)   Interrupt pin  //não sei se necessario


  MPU6050  UNO          Description
  ======= ==========    ====================================================
  VCC     VU (5V)       Power
  GND     G             Ground
  SCL     A5            I2C clock
  SDA     A4            I2C data
  XDA     not connected
  XCL     not connected
  AD0     not connected
  INT     D2            Interrupt pin  //não sei se necessario

*/

// só usa a internet se for NodeMCU
#if defined(ESP8266)
#include <ESP8266WiFi.h>
#include <WiFiClient.h>
#include <ESP8266WebServer.h>
#include <WiFiUdp.h>

#ifndef APSSID
#define APSSID "rede_gyro"
#define APPSK ""
#define UDP_PORT 5555
#endif

/* Set these to your desired credentials. */
const char *ssid = APSSID;
const char *password = APPSK;

// UDP
WiFiUDP UDP;
#endif

#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>
#include <Wire.h>
/*
  Os identificadores atualmente reconhecidos são
  papE, papD, luvE e luvD.
*/
String identificador = "papE"; // papete direita

Adafruit_MPU6050 mpu;
sensors_event_t a, g, temp;

double pitch, roll; // Roll & Pitch são os angulos dos eixos X e Y

void setup()
{
    Serial.begin(9600);

    // Tenta iniciar MPU
    if (!mpu.begin())
    {
        Serial.println("Falha ao localizar MPU6050 (as conexões estão certas?)");
        while (1)
            delay(10);
    }

    // Configurações que não entendo
    mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
    mpu.setGyroRange(MPU6050_RANGE_500_DEG);
    mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);

    //delay para aplicar as configurações
    delay(100);

    // obtem offset inicial
    mpu.getEvent(&a, &g, &temp);

// setup rede
#if defined(ESP8266)
    WiFi.softAP(ssid, password);
#endif
}

void loop()
{
    // lê dados do sensor
    mpu.getEvent(&a, &g, &temp);

    // calcula angulos em radianos
    pitch = atan2((-a.acceleration.x), sqrt(a.acceleration.y * a.acceleration.y + a.acceleration.z * a.acceleration.z));
    roll = atan2(a.acceleration.y, a.acceleration.z);

    String msg = identificador + '\t' + String(pitch, 6) + '\t' + String(roll, 6);

    Serial.println(msg);
#if defined(ESP8266)
    if (WiFi.softAPgetStationNum() > 0)
    {
        // Se o computador tiver se conectado a rede, e somente ele, este será o endereço IP atribuido
        UDP.beginPacket("192.168.4.2", 5555); 
        UDP.println(msg);                     // Adiciona-se o valor ao pacote.
        UDP.endPacket();
    }
#endif
    delay(50);
}