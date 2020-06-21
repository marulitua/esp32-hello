use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

use esp_idf_bindgen::{esp_mac_type_t, esp_read_mac};
use macaddr::{MacAddr, MacAddr6};

use super::assert_esp_ok;

/// Enumeration of all available interfaces.
#[derive(Debug, Clone, Copy)]
pub enum Interface {
  /// WiFi interface in station mode.
  Sta,
  /// WiFi interface in access point mode.
  Ap,
  #[cfg(not(target_device = "esp8266"))]
  /// Bluetooth interface.
  Bt,
  /// Ethernet interface.
  #[cfg(not(target_device = "esp8266"))]
  Eth,
}

/// ```no_run
/// use macaddr::MacAddr6;
/// use esp32_hal::Interface;
///
/// MacAddr6::from(Interface::Ap)
/// ```
impl From<Interface> for MacAddr6 {
  fn from(interface: Interface) -> Self {
    let mac_address_type = match interface {
      Interface::Sta => esp_mac_type_t::ESP_MAC_WIFI_STA,
      Interface::Ap  => esp_mac_type_t::ESP_MAC_WIFI_SOFTAP,
      #[cfg(not(target_device = "esp8266"))]
      Interface::Bt  => esp_mac_type_t::ESP_MAC_BT,
      #[cfg(not(target_device = "esp8266"))]
      Interface::Eth => esp_mac_type_t::ESP_MAC_ETH,
    };

    let mut mac_address = MaybeUninit::<Self>::uninit();
    assert_esp_ok!(esp_read_mac(mac_address.as_mut_ptr() as *mut _, mac_address_type));
    unsafe { mac_address.assume_init() }
  }
}

/// ```no_run
/// use macaddr::MacAddr;
/// use esp32_hal::Interface;
///
/// MacAddr::from(Interface::Ap)
/// ```
impl From<Interface> for MacAddr {
  fn from(interface: Interface) -> Self {
    Self::V6(interface.into())
  }
}

#[derive(Debug)]
pub struct IpInfo {
  ip: Ipv4Addr,
  netmask: Ipv4Addr,
  gateway: Ipv4Addr,
}

impl IpInfo {
  pub fn ip(&self) -> &Ipv4Addr {
    &self.ip
  }

  pub fn netmask(&self) -> &Ipv4Addr {
    &self.netmask
  }

  pub fn gateway(&self) -> &Ipv4Addr {
    &self.gateway
  }
}

#[cfg(target_device = "esp8266")]
use esp_idf_bindgen::{tcpip_adapter_get_ip_info, tcpip_adapter_if_t, tcpip_adapter_ip_info_t as ip_info_t};

#[cfg(target_device = "esp32")]
use esp_idf_bindgen::{esp_netif_get_ip_info, esp_netif_get_handle_from_ifkey, esp_netif_ip_info_t as ip_info_t};

impl IpInfo {
  #[cfg(target_device = "esp8266")]
  pub fn sta() -> Option<Self> {
    Self::get_ip_info(tcpip_adapter_if_t::TCPIP_ADAPTER_IF_STA)
  }

  #[cfg(target_device = "esp32")]
  pub fn sta() -> Option<Self> {
    Self::get_ip_info(b"WIFI_STA_DEF\0")
  }

  #[cfg(target_device = "esp8266")]
  pub fn ap() -> Option<Self> {
    Self::get_ip_info(tcpip_adapter_if_t::TCPIP_ADAPTER_IF_AP)
  }

  #[cfg(target_device = "esp32")]
  pub fn ap() -> Option<Self> {
    Self::get_ip_info(b"WIFI_AP_DEF\0")
  }

  pub(crate) unsafe fn from_native_unchecked(ip_info: ip_info_t) -> Self {
    IpInfo {
      ip: u32::from_be(ip_info.ip.addr).into(),
      netmask: u32::from_be(ip_info.netmask.addr).into(),
      gateway: u32::from_be(ip_info.gw.addr).into(),
    }
  }

  pub(crate) fn from_native(ip_info: ip_info_t) -> Option<Self> {
    if ip_info.ip.addr == 0 && ip_info.netmask.addr == 0 && ip_info.gw.addr == 0 {
      return None;
    }

    let ip = u32::from_be(ip_info.ip.addr);
    let netmask = u32::from_be(ip_info.netmask.addr);
    let gateway = u32::from_be(ip_info.gw.addr);

    if ip == 0 && netmask == 0 && gateway == 0 {
      return None;
    }

    Some(unsafe { Self::from_native_unchecked(ip_info) })
  }

  #[cfg(target_device = "esp8266")]
  fn get_ip_info(interface: tcpip_adapter_if_t) -> Option<Self> {

    let mut ip_info = MaybeUninit::<ip_info_t>::uninit();
    assert_esp_ok!(tcpip_adapter_get_ip_info(interface, ip_info.as_mut_ptr()));
    Self::from_native(unsafe { ip_info.assume_init() })
  }

  #[cfg(target_device = "esp32")]
  fn get_ip_info(key: &[u8]) -> Option<Self> {
    let mut ip_info = MaybeUninit::<ip_info_t>::uninit();
    let interface = unsafe { esp_netif_get_handle_from_ifkey(key.as_ptr() as *const _) };
    assert_esp_ok!(esp_netif_get_ip_info(interface, ip_info.as_mut_ptr()));
    Self::from_native(unsafe { ip_info.assume_init() })
  }
}