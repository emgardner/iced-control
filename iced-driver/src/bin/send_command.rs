use iced_driver::DeviceDriver;
use tokio;
use tokio::time::{sleep, Duration};
use tokio_serial::SerialPortBuilderExt;

#[tokio::main]
async fn main() {
    println!("Here");
    let port = tokio_serial::new("/dev/ttyACM0", 115200)
        .open_native_async()
        .unwrap();
    let mut driver = DeviceDriver::new(port);
    loop {
        println!("Running");
        driver.set_gpio().await;
        sleep(Duration::from_millis(1000)).await;
        driver.clear_gpio().await;
        sleep(Duration::from_millis(1000)).await;
    }
}
