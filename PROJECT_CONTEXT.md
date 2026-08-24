# Project Context: quadromon

## Project Vision & Goal

Ziel von des Quaadromon ist es eine Randlose always on Top UI zur Visualisierung bestimmter Hardware Sensoren
auszulesen.
Die Hardware Sensor informationen werden durch das OS mittels lm_sensors bereitgestellt. Quadromon stellt neben den
aktuellen Sensorinformationen, statistische Daten als 512 Bins ueber alle Messungen zur Verfuegung.

- **Core Objective:** Track Quality of the Watercooling System over Time.
- **Target Audience:** Me and other linux Users with watercooling systems.
- **Key Success Criteria:** Provide a clear and intuitive visualization of watercooling system performance over time.

## Architecture & Tech Stack

Die Visualisierung erfolgt mittels vello, und der gesamte UI Stack ist im vello subproject. Das quadrosrv subproject
liest die Sensorinformation, berechnet statistiken und serialisiert historische Daten. Allgemein sollen die Menge an
Abhaengigkeiten so gering wie moeglich sein.

### Tech Stack

- **Language:** Rust
- **UI Framework:** Vello
- **Data Serialization:** Dateibasiert in home Folder in .quadro.
- **Dependency Management:** Cargo

### Project Structure

- **/examples:** Demo and Example code.
- **/fluxo:** Contains the UI stack and visualization components.
- **/quadrosrv:** Handles sensor data reading, statistical calculations, and historical data serialization.

## Development Workflow & Rules

- **Code Formatting** "Verwende clippy."
- **Testing Rules** "Erstelle Tests fuer jede pub Function die du in /quadromon hinzufuegst.
- **Error Handling** "Alle Functionen die existentiell sind, sollten mit einer panic failen. Jede Fehlerbehandlung soll
  in der Fehlercondition gelogged werden."
- **Documentation** "Wenn du neue Feature implementierst, lege ein Expample Project dazu an, versuche an sonsten
  ausschliesslich inline Kommentare zu machen, sei damit aber sparsam."
- **Refactoring** "Wenn du vorhandenen Funktionen in /quadrosrv refactoren willst, erstelle eine Kopie der alten
  Function mit dem Suffix _old. Ueberpruefe dann bevor du aenderungen machst, ob Tests existieren und passe diese
  Entsprechend an. Wenn du mit dem Refactoring fertig bist und es Tests gab, erstelle neue vergleichbare Tests."
- **Serialisierung** "Verwende JSON im home Folder in .quadro."

## Current Roadmap / Next Steps

- Finish implementierung der App Config.
- Speichern historische Daten.
- 