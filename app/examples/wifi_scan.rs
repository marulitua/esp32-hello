#![no_main]

use std::time::Duration;

use futures::executor::block_on;

use esp_idf_hal::{nvs::*, wifi::*};

#[no_mangle]
fn app_main() {
  block_on(async {
    let mut nvs = NonVolatileStorage::default();
    let mut wifi = Wifi::init(&mut nvs).unwrap();

    let scan_config = ScanConfig::builder()
      .show_hidden(true)
      .scan_type(ScanType::Passive { max: Duration::from_secs(30) })
      .build();

    loop {
      match wifi.scan(&scan_config).await {
        Ok(aps) => {
          if aps.is_empty() {
            println!("No access points found.");
          } else {
            println!("Found {} access points:", aps.len());

            for ap in aps {
              println!("  - {} '{}'", ap.bssid(), ap.ssid())
            }
          }
        }
        Err(err) => {
          eprintln!("WiFi Scan failed: {}", err);
        }
      }
    }
  })
}
