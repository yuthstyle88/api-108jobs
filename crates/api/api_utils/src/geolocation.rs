use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};

/// Hard cap for geolocation lookups. Used to be unbounded; an unresponsive
/// third-party would stall request threads. ip-api responds in <100ms on a
/// healthy day; 4s is enough headroom without risking client-perceived
/// latency.
const GEOLOCATION_TIMEOUT: Duration = Duration::from_secs(4);
const GEOLOCATION_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
struct IpApiResponse {
  country: String,
  #[serde(rename = "countryCode")]
  country_code: String,
  status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryInfo {
  pub name: String,
  pub code: String,
}

pub struct GeolocationService {
  client: Client,
}

impl GeolocationService {
  pub fn new() -> Self {
    let client = Client::builder()
      .timeout(GEOLOCATION_TIMEOUT)
      .connect_timeout(GEOLOCATION_CONNECT_TIMEOUT)
      .build()
      .unwrap_or_else(|_| Client::new());
    Self { client }
  }

  /// Detect country from IP address using ip-api.com (free tier)
  pub async fn detect_country_from_ip(&self, ip: IpAddr) -> FastJobResult<CountryInfo> {
    // Skip detection for local/private IPs
    let is_private = match ip {
      IpAddr::V4(v4) => v4.is_private(),
      IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local(),
    };
    if ip.is_loopback() || is_private {
      return Ok(CountryInfo {
        name: "Thailand".to_string(), // Default for development
        code: "TH".to_string(),
      });
    }

    let url = format!("http://ip-api.com/json/{}", ip);

    let response = self
      .client
      .get(&url)
      .send()
      .await
      .map_err(|e| FastJobErrorType::InvalidField(format!("Geolocation API error: {}", e)))?;

    let geo_data: IpApiResponse = response.json().await.map_err(|e| {
      FastJobErrorType::InvalidField(format!("Failed to parse geolocation response: {}", e))
    })?;

    if geo_data.status != "success" {
      return Err(FastJobErrorType::InvalidField(
        "Geolocation detection failed".to_string(),
      ))?;
    }

    // Map country names to our supported regions
    let normalized_country = match geo_data.country.as_str() {
      "Thailand" => "Thailand",
      "Vietnam" => "Vietnam",
      // Default to Thailand for other countries (or could show error)
      _ => "Thailand",
    };

    Ok(CountryInfo {
      name: normalized_country.to_string(),
      code: geo_data.country_code,
    })
  }

  /// Alternative method using a different free IP geolocation service
  pub async fn detect_country_from_ip_alt(&self, ip: IpAddr) -> FastJobResult<CountryInfo> {
    let is_private = match ip {
      IpAddr::V4(v4) => v4.is_private(),
      IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local(),
    };
    if ip.is_loopback() || is_private {
      return Ok(CountryInfo {
        name: "Thailand".to_string(),
        code: "TH".to_string(),
      });
    }

    // Using ipapi.co as alternative (also free)
    let url = format!("https://ipapi.co/{}/json/", ip);

    #[derive(Deserialize)]
    struct IpapiResponse {
      country_name: String,
      country_code: String,
    }

    let response = self
      .client
      .get(&url)
      .header("User-Agent", "FastWork/1.0")
      .send()
      .await
      .map_err(|e| FastJobErrorType::InvalidField(format!("Geolocation API error: {}", e)))?;

    let geo_data: IpapiResponse = response.json().await.map_err(|e| {
      FastJobErrorType::InvalidField(format!("Failed to parse geolocation response: {}", e))
    })?;

    let normalized_country = match geo_data.country_name.as_str() {
      "Thailand" => "Thailand",
      "Vietnam" => "Vietnam",
      _ => "Thailand", // Default
    };

    Ok(CountryInfo {
      name: normalized_country.to_string(),
      code: geo_data.country_code,
    })
  }

  /// Validate if detected country is supported
  pub fn is_supported_country(country: &str) -> bool {
    matches!(country, "Thailand" | "Vietnam")
  }
}

impl Default for GeolocationService {
  fn default() -> Self {
    Self::new()
  }
}
