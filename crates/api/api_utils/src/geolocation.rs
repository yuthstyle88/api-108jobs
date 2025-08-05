use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use lemmy_utils::error::{FastJobResult, FastJobErrorType};

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
        Self {
            client: Client::new(),
        }
    }

    /// Detect country from IP address using ip-api.com (free tier)
    pub async fn detect_country_from_ip(&self, ip: IpAddr) -> FastJobResult<CountryInfo> {
        // Skip detection for local/private IPs
        if ip.is_loopback() || ip.is_private() {
            return Ok(CountryInfo {
                name: "Thailand".to_string(), // Default for development
                code: "TH".to_string(),
            });
        }

        let url = format!("http://ip-api.com/json/{}", ip);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| FastJobErrorType::InvalidField(format!("Geolocation API error: {}", e)))?;

        let geo_data: IpApiResponse = response
            .json()
            .await
            .map_err(|e| FastJobErrorType::InvalidField(format!("Failed to parse geolocation response: {}", e)))?;

        if geo_data.status != "success" {
            return Err(FastJobErrorType::InvalidField("Geolocation detection failed".to_string()))?;
        }

        // Map country names to our supported regions
        let normalized_country = match geo_data.country.as_str() {
            "Thailand" => "Thailand",
            "Vietnam" => "Vietnam", 
            // Default to Thailand for other countries (or could show error)
            _ => "Thailand"
        };

        Ok(CountryInfo {
            name: normalized_country.to_string(),
            code: geo_data.country_code,
        })
    }

    /// Alternative method using a different free IP geolocation service
    pub async fn detect_country_from_ip_alt(&self, ip: IpAddr) -> FastJobResult<CountryInfo> {
        if ip.is_loopback() || ip.is_private() {
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

        let response = self.client
            .get(&url)
            .header("User-Agent", "FastWork/1.0")
            .send()
            .await
            .map_err(|e| FastJobErrorType::InvalidField(format!("Geolocation API error: {}", e)))?;

        let geo_data: IpapiResponse = response
            .json()
            .await
            .map_err(|e| FastJobErrorType::InvalidField(format!("Failed to parse geolocation response: {}", e)))?;

        let normalized_country = match geo_data.country_name.as_str() {
            "Thailand" => "Thailand",
            "Vietnam" => "Vietnam",
            _ => "Thailand" // Default
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