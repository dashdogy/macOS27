use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct IORegistryDiagnostic {
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Diagnostics {
    #[serde(rename = "IORegistry")]
    pub ioregistry: IORegistry,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AdapterDetails {
    pub adapter_voltage: Option<i32>,
    pub is_wireless: Option<bool>,
    pub watts: Option<i32>,
    pub name: Option<String>,
    pub current: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PowerTelemetryData {
    pub adapter_efficiency_loss: i32,
    pub battery_power: i64,
    pub system_current_in: i32,
    pub system_energy_consumed: i64,
    pub system_load: i64,
    pub system_power_in: i32,
    pub system_voltage_in: i32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct IORegistry {
    pub adapter_details: AdapterDetails,
    pub power_telemetry_data: Option<PowerTelemetryData>,
    pub absolute_capacity: i32,
    pub amperage: i32,
    pub voltage: i32,
    pub apple_raw_battery_voltage: Option<i32>,
    pub apple_raw_current_capacity: i32,
    pub apple_raw_max_capacity: i32,
    pub current_capacity: i32,
    pub cycle_count: i32,
    pub design_capacity: i32,
    pub fully_charged: bool,
    pub instant_amperage: i32,
    pub is_charging: bool,
    pub max_capacity: i32,
    pub temperature: i32,
    pub time_remaining: i32,
    // TODO: check
    pub update_time: i64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IORegistryDataError {
    #[error("missing required battery capacity `{0}`")]
    MissingCapacity(&'static str),
    #[error("battery capacity `{field}` must be positive, got {value}")]
    NonPositiveCapacity { field: &'static str, value: i32 },
}

pub(crate) mod repr {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all(deserialize = "PascalCase"))]
    pub struct IORegistryDiagnostic {
        pub diagnostics: Diagnostics,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all(deserialize = "PascalCase"))]
    pub struct Diagnostics {
        #[serde(rename = "IORegistry")]
        pub ioregistry: IORegistry,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default, rename_all(deserialize = "PascalCase"))]
    pub struct AdapterDetails {
        pub adapter_voltage: Option<i32>,
        pub is_wireless: Option<bool>,
        pub watts: Option<i32>,
        pub name: Option<String>,
        pub current: Option<i32>,
        pub description: Option<String>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default, rename_all(deserialize = "PascalCase"))]
    pub struct PowerTelemetryData {
        pub adapter_efficiency_loss: i32,
        pub battery_power: i64,
        pub system_current_in: i32,
        pub system_energy_consumed: i64,
        pub system_load: i64,
        pub system_power_in: i32,
        pub system_voltage_in: i32,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default, rename_all(deserialize = "PascalCase"))]
    pub struct BatteryData {
        pub absolute_capacity: Option<i32>,
        pub remaining_capacity: Option<i32>,
        pub full_charge_capacity: Option<i32>,
        pub design_capacity: Option<i32>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default, rename_all(deserialize = "PascalCase"))]
    pub struct IORegistry {
        pub adapter_details: AdapterDetails,
        pub power_telemetry_data: Option<PowerTelemetryData>,
        pub battery_data: BatteryData,
        pub absolute_capacity: Option<i32>,
        pub amperage: i32,
        pub voltage: i32,
        pub apple_raw_battery_voltage: Option<i32>,
        pub apple_raw_current_capacity: Option<i32>,
        pub apple_raw_max_capacity: Option<i32>,
        pub current_capacity: i32,
        pub cycle_count: i32,
        pub design_capacity: Option<i32>,
        pub fully_charged: bool,
        pub instant_amperage: i32,
        pub is_charging: bool,
        pub max_capacity: i32,
        pub temperature: i32,
        pub time_remaining: i32,
        pub update_time: i64,
    }
}

impl From<repr::AdapterDetails> for AdapterDetails {
    fn from(value: repr::AdapterDetails) -> Self {
        Self {
            adapter_voltage: value.adapter_voltage,
            is_wireless: value.is_wireless,
            watts: value.watts,
            name: value.name,
            current: value.current,
            description: value.description,
        }
    }
}

impl From<repr::PowerTelemetryData> for PowerTelemetryData {
    fn from(value: repr::PowerTelemetryData) -> Self {
        Self {
            adapter_efficiency_loss: value.adapter_efficiency_loss,
            battery_power: value.battery_power,
            system_current_in: value.system_current_in,
            system_energy_consumed: value.system_energy_consumed,
            system_load: value.system_load,
            system_power_in: value.system_power_in,
            system_voltage_in: value.system_voltage_in,
        }
    }
}

impl TryFrom<repr::IORegistry> for IORegistry {
    type Error = IORegistryDataError;

    fn try_from(value: repr::IORegistry) -> Result<Self, Self::Error> {
        let raw_current_capacity = value
            .apple_raw_current_capacity
            .or(value.battery_data.remaining_capacity)
            .ok_or(IORegistryDataError::MissingCapacity(
                "AppleRawCurrentCapacity or BatteryData.RemainingCapacity",
            ))?;

        let raw_max_capacity = value
            .apple_raw_max_capacity
            .or(value.battery_data.full_charge_capacity)
            .ok_or(IORegistryDataError::MissingCapacity(
                "AppleRawMaxCapacity or BatteryData.FullChargeCapacity",
            ))?;
        if raw_max_capacity <= 0 {
            return Err(IORegistryDataError::NonPositiveCapacity {
                field: "AppleRawMaxCapacity or BatteryData.FullChargeCapacity",
                value: raw_max_capacity,
            });
        }

        let design_capacity = value
            .design_capacity
            .or(value.battery_data.design_capacity)
            .ok_or(IORegistryDataError::MissingCapacity(
                "DesignCapacity or BatteryData.DesignCapacity",
            ))?;
        if design_capacity <= 0 {
            return Err(IORegistryDataError::NonPositiveCapacity {
                field: "DesignCapacity or BatteryData.DesignCapacity",
                value: design_capacity,
            });
        }

        Ok(Self {
            adapter_details: value.adapter_details.into(),
            power_telemetry_data: value.power_telemetry_data.map(Into::into),
            absolute_capacity: value
                .absolute_capacity
                .or(value.battery_data.absolute_capacity)
                .unwrap_or_default(),
            amperage: value.amperage,
            voltage: value.voltage,
            apple_raw_battery_voltage: value.apple_raw_battery_voltage,
            apple_raw_current_capacity: raw_current_capacity,
            apple_raw_max_capacity: raw_max_capacity,
            current_capacity: value.current_capacity,
            cycle_count: value.cycle_count,
            design_capacity,
            fully_charged: value.fully_charged,
            instant_amperage: value.instant_amperage,
            is_charging: value.is_charging,
            max_capacity: value.max_capacity,
            temperature: value.temperature,
            time_remaining: value.time_remaining,
            update_time: value.update_time,
        })
    }
}

impl TryFrom<repr::Diagnostics> for Diagnostics {
    type Error = IORegistryDataError;

    fn try_from(value: repr::Diagnostics) -> Result<Self, Self::Error> {
        Ok(Self {
            ioregistry: value.ioregistry.try_into()?,
        })
    }
}

impl TryFrom<repr::IORegistryDiagnostic> for IORegistryDiagnostic {
    type Error = IORegistryDataError;

    fn try_from(value: repr::IORegistryDiagnostic) -> Result<Self, Self::Error> {
        Ok(Self {
            diagnostics: value.diagnostics.try_into()?,
        })
    }
}

impl Deref for IORegistry {
    type Target = Option<PowerTelemetryData>;

    fn deref(&self) -> &Self::Target {
        &self.power_telemetry_data
    }
}

impl IORegistry {
    pub fn ptd(&self) -> Option<&PowerTelemetryData> {
        self.power_telemetry_data.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{repr, IORegistry, IORegistryDataError, IORegistryDiagnostic};

    fn plist_document(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>{body}</dict></plist>"#
        )
    }

    fn parse_ioreg(body: &str) -> repr::IORegistry {
        plist::from_bytes(plist_document(body).as_bytes()).expect("valid IORegistry test plist")
    }

    #[test]
    fn converts_legacy_flat_battery_capacities() {
        let raw = parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4100</integer>
<key>AppleRawMaxCapacity</key><integer>5200</integer>
<key>DesignCapacity</key><integer>6100</integer>
<key>AbsoluteCapacity</key><integer>67</integer>
<key>CurrentCapacity</key><integer>79</integer>
<key>CycleCount</key><integer>240</integer>
"#,
        );

        let converted = IORegistry::try_from(raw).expect("legacy schema should convert");
        assert_eq!(converted.apple_raw_current_capacity, 4100);
        assert_eq!(converted.apple_raw_max_capacity, 5200);
        assert_eq!(converted.design_capacity, 6100);
        assert_eq!(converted.absolute_capacity, 67);
        assert_eq!(converted.current_capacity, 79);
        assert_eq!(converted.cycle_count, 240);
    }

    #[test]
    fn converts_macos_27_nested_battery_capacities() {
        let raw = parse_ioreg(
            r#"
<key>BatteryData</key><dict>
  <key>AbsoluteCapacity</key><integer>88</integer>
  <key>RemainingCapacity</key><integer>5437</integer>
  <key>FullChargeCapacity</key><integer>5492</integer>
  <key>DesignCapacity</key><integer>6249</integer>
</dict>
<key>Amperage</key><integer>439</integer>
<key>Voltage</key><integer>13106</integer>
<key>CurrentCapacity</key><integer>100</integer>
<key>AdapterDetails</key><dict>
  <key>Watts</key><integer>100</integer>
  <key>AdapterVoltage</key><integer>20000</integer>
</dict>
<key>PowerTelemetryData</key><dict>
  <key>BatteryPower</key><integer>5674</integer>
  <key>SystemPowerIn</key><integer>38272</integer>
</dict>
"#,
        );

        let converted = IORegistry::try_from(raw).expect("macOS 27 schema should convert");
        assert_eq!(converted.apple_raw_current_capacity, 5437);
        assert_eq!(converted.apple_raw_max_capacity, 5492);
        assert_eq!(converted.design_capacity, 6249);
        assert_eq!(converted.absolute_capacity, 88);
        assert_eq!(converted.amperage, 439);
        assert_eq!(converted.voltage, 13106);
        assert_eq!(converted.adapter_details.watts, Some(100));
        assert_eq!(converted.adapter_details.adapter_voltage, Some(20000));
        assert_eq!(converted.ptd().map(|data| data.battery_power), Some(5674));
        assert_eq!(
            converted.ptd().map(|data| data.system_power_in),
            Some(38272)
        );
    }

    #[test]
    fn legacy_flat_capacities_take_precedence_over_nested_values() {
        let raw = parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4001</integer>
<key>AppleRawMaxCapacity</key><integer>5001</integer>
<key>DesignCapacity</key><integer>6001</integer>
<key>BatteryData</key><dict>
  <key>RemainingCapacity</key><integer>4002</integer>
  <key>FullChargeCapacity</key><integer>5002</integer>
  <key>DesignCapacity</key><integer>6002</integer>
</dict>
"#,
        );

        let converted = IORegistry::try_from(raw).expect("combined schema should convert");
        assert_eq!(converted.apple_raw_current_capacity, 4001);
        assert_eq!(converted.apple_raw_max_capacity, 5001);
        assert_eq!(converted.design_capacity, 6001);
    }

    #[test]
    fn rejects_missing_or_non_positive_essential_capacities() {
        let missing = IORegistry::try_from(parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4000</integer>
<key>DesignCapacity</key><integer>6000</integer>
"#,
        ))
        .expect_err("missing maximum capacity should fail");
        assert!(matches!(
            missing,
            IORegistryDataError::MissingCapacity(
                "AppleRawMaxCapacity or BatteryData.FullChargeCapacity"
            )
        ));

        let missing_design = IORegistry::try_from(parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4000</integer>
<key>AppleRawMaxCapacity</key><integer>5000</integer>
"#,
        ))
        .expect_err("missing design capacity should fail");
        assert!(matches!(
            missing_design,
            IORegistryDataError::MissingCapacity("DesignCapacity or BatteryData.DesignCapacity")
        ));

        let zero_max = IORegistry::try_from(parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4000</integer>
<key>AppleRawMaxCapacity</key><integer>0</integer>
<key>DesignCapacity</key><integer>6000</integer>
"#,
        ))
        .expect_err("zero maximum capacity should fail");
        assert_eq!(
            zero_max,
            IORegistryDataError::NonPositiveCapacity {
                field: "AppleRawMaxCapacity or BatteryData.FullChargeCapacity",
                value: 0,
            }
        );

        let zero_design = IORegistry::try_from(parse_ioreg(
            r#"
<key>AppleRawCurrentCapacity</key><integer>4000</integer>
<key>AppleRawMaxCapacity</key><integer>5000</integer>
<key>DesignCapacity</key><integer>0</integer>
"#,
        ))
        .expect_err("zero design capacity should fail");
        assert_eq!(
            zero_design,
            IORegistryDataError::NonPositiveCapacity {
                field: "DesignCapacity or BatteryData.DesignCapacity",
                value: 0,
            }
        );
    }

    #[test]
    fn safely_converts_remote_diagnostic_wrapper() {
        let xml = plist_document(
            r#"
<key>Diagnostics</key><dict>
  <key>IORegistry</key><dict>
    <key>AppleRawCurrentCapacity</key><integer>2100</integer>
    <key>AppleRawMaxCapacity</key><integer>3200</integer>
    <key>DesignCapacity</key><integer>3800</integer>
  </dict>
</dict>
"#,
        );
        let raw: repr::IORegistryDiagnostic =
            plist::from_bytes(xml.as_bytes()).expect("valid remote diagnostic plist");

        let converted =
            IORegistryDiagnostic::try_from(raw).expect("remote diagnostic should convert safely");
        assert_eq!(
            converted.diagnostics.ioregistry.apple_raw_current_capacity,
            2100
        );
        assert_eq!(
            converted.diagnostics.ioregistry.apple_raw_max_capacity,
            3200
        );
    }
}
