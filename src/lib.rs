pub mod codesys {
    use rtu_client::{Client, Context};
    use serde::{Deserialize, Serialize};
    use sqlx::sqlite::SqlitePool;
    use std::time::Duration;
    use tokio::time;
    use tokio_serial::SerialPortBuilderExt;
    use plotters::prelude::*;
    use chrono::prelude::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MotorSpecs {
        pub rated_power: f64, // kW
        pub rated_torque: f64, // Nm
        pub rated_speed: f64, // rpm
        pub peak_torque: f64, // Nm
        pub max_speed: f64, // rpm
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MotorData {
        pub timestamp: i64,
        pub current_power: f64,
        pub current_torque: f64,
        pub current_speed: f64,
        pub current_heat: f64,
        pub current_cycles: f64,
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

    pub async fn insert_motor_data(pool: &SqlitePool, data: &MotorData) {
        sqlx::query!(
            r#"
            INSERT INTO motor_data (timestamp, current_power, current_torque, current_speed, current_heat, current_cycles)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            data.timestamp,
            data.current_power,
            data.current_torque,
            data.current_speed,
            data.current_heat,
            data.current_cycles
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub async fn setup_database() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite://motor_data.db").await.unwrap();
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS motor_data (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                current_power REAL NOT NULL,
                current_torque REAL NOT NULL,
                current_speed REAL NOT NULL,
                current_heat REAL NOT NULL,
                current_cycles REAL NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    pub fn draw_chart(filename: &str, data: &[(i64, f64)], title: &str, x_label: &str, y_label: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(data.first().unwrap().0..data.last().unwrap().0, 0.0..data.iter().map(|d| d.1).fold(0.0 / 0.0, f64::max))?;

        chart.configure_mesh().x_desc(x_label).y_desc(y_label).draw()?;
        chart.draw_series(LineSeries::new(
            data.iter().map(|(x, y)| (*x, *y)),
            &RED,
        ))?;

        Ok(())
    }

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

    fn calculate_power(volts: f64, amps: f64) -> f64 {
        volts * amps / 1000.0 // Convert to kW
    }

    fn calculate_cycles(torque: f64, period: f64) -> f64 {
        torque * period
    }

    pub async fn run_motor_monitoring() {
        // Example motor specification (EY630EAK)
        let motor = MotorSpecs::new(2.4, 10.1, 1450.0, 25.9, 4800.0);
        let pool = setup_database().await;
        let pool = Arc::new(pool);

        // Set up Modbus RTU connection
        let serial_port = tokio_serial::new("/dev/ttyUSB0", 9600)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .flow_control(tokio_serial::FlowControl::None)
            .open_native_async()
            .unwrap();

        let mut ctx = Client::new(serial_port, 1);

        let mut interval = time::interval(Duration::from_secs(1));
        let motor_data = Arc::new(Mutex::new(Vec::new()));

        loop {
            interval.tick().await;

            let data = read_modbus_data(&mut ctx).await;

            let mut motor_data_lock = motor_data.lock().await;
            motor_data_lock.push((data.timestamp, data.current_power));
            motor_data_lock.push((data.timestamp, data.current_torque));
            motor_data_lock.push((data.timestamp, data.current_speed));
            motor_data_lock.push((data.timestamp, data.current_heat));
            motor_data_lock.push((data.timestamp, data.current_cycles));

            let pool = Arc::clone(&pool);
            tokio::spawn(async move {
                insert_motor_data(&pool, &data).await;
            });

            // Update graphs
            draw_chart("current_power.png", &motor_data_lock, "Current Power", "Time", "Power (kW)").unwrap();
            draw_chart("current_torque.png", &motor_data_lock, "Current Torque", "Time", "Torque (Nm)").unwrap();
            draw_chart("current_speed.png", &motor_data_lock, "Current Speed", "Time", "Speed (rpm)").unwrap();
            draw_chart("current_heat.png", &motor_data_lock, "Current Heat", "Time", "Heat (°C)").unwrap();
            draw_chart("current_cycles.png", &motor_data_lock, "Current Cycles", "Time", "Cycles (Nm.s)").unwrap();
        }
    }
}
