# Rustys Library

The rustys library provides functions to interact with CODESYS controllers and handle motor data.

## Features

- Reads data from the CODESYS controller using Modbus RTU.
- Stores motor data in an SQLite database.
- Generates real-time charts using the plotters library.

## Functions

### calculate_power

Calculates power from voltage and current.

### calculate_cycles

Calculates cycles based on torque and period.

### insert_motor_data

Inserts motor data into an SQLite database.

### setup_database

Sets up the SQLite database.

### draw_chart

Draws real-time charts using the plotters library.

### read_modbus_data

Reads data from Modbus registers.

### run_motor_monitoring

Main function to monitor the motor, read data, store it, and update the charts.

## Configuration

Ensure your CODESYS PLC is set up to communicate over Modbus RTU and adjust the Modbus register addresses in the `read_modbus_data` function to match your PLC configuration.

## Example

The rustys library provides functions to interact with CODESYS controllers and handle motor data. Below is a brief overview of the main components:

### MotorSpecs

Holds motor specifications:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct MotorSpecs {
    pub rated_power: f64, // kW
    pub rated_torque: f64, // Nm
    pub rated_speed: f64, // rpm
    pub peak_torque: f64, // Nm
    pub max_speed: f64, // rpm
}

impl MotorSpecs {
    pub fn new(rated_power: f64, rated_torque: f64, rated_speed: f64, peak_torque: f64, max_speed: f64) -> Self {
        MotorSpecs {
            rated_power,
            rated_torque,
            rated_speed,
            peak_torque,
            max_speed,
        }
    }
}
```

### MotorData
Holds motor data readings:
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct MotorData {
    pub timestamp: i64,
    pub current_power: f64,
    pub current_torque: f64,
    pub current_speed: f64,
    pub current_heat: f64,
    pub current_cycles: f64,
}
```
## Configuration
Make sure that your Codesys PLC is set up to communicate over Modbus RTU and adjust the Modbus register addresses in the read_modbus_data function to match your PLC configuration:
```rust
pub async fn read_modbus_data(ctx: &mut Client) -> MotorData {
    let voltage_reading = ctx.read_input_registers(0, 1).await.unwrap()[0] as f64;
    let current_reading = ctx.read_input_registers(1, 1).await.unwrap()[0] as f64;
    let heat_reading = ctx.read_input_registers(2, 1).await.unwrap()[0] as f64;
    let speed_reading = ctx.read_input_registers(3, 1).await.unwrap()[0] as f64;
    let period = 1.0; // Example period

    let current_power = calculate_power(voltage_reading, current_reading);
    let current_torque = 10.1; // Example value
    let current_cycles = calculate_cycles(current_torque, period);

    let now = Local::now().timestamp();

    MotorData {
        timestamp: now,
        current_power,
        current_torque,
        current_speed: speed_reading,
        current_heat: heat_reading,
        current_cycles,
    }
}
```