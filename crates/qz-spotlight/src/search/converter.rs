//! # Unit and Currency Conversion Subsystem
//!
//! Natural language conversion between scientific, imperial, digital, and monetary units.
//!
//! Supports queries such as:
//! - `100 km to mi`
//! - `72 F in C`
//! - `50 USD in EUR`
//! - `16 GB to MiB`
//! - `2.5 kg to lbs`

use std::collections::HashMap;
use thiserror::Error;

/// Conversion evaluation errors.
#[derive(Debug, Error, PartialEq)]
pub enum ConvertError {
    /// Input string did not match conversion pattern.
    #[error("Invalid conversion format. Expected '<amount> <from_unit> [to|in] <to_unit>'")]
    InvalidFormat,
    /// Unknown unit symbol or name.
    #[error("Unknown unit: '{0}'")]
    UnknownUnit(String),
    /// Units belong to incompatible physical dimensions (e.g. converting km to kg).
    #[error("Incompatible unit dimensions: cannot convert '{0}' to '{1}'")]
    IncompatibleDimensions(String, String),
    /// Invalid numeric quantity.
    #[error("Invalid number value: '{0}'")]
    InvalidAmount(String),
}

/// Physical or monetary dimension category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitDimension {
    /// Linear distance and length.
    Length,
    /// Mass and weight.
    Mass,
    /// Thermodynamic temperature.
    Temperature,
    /// Digital computer memory / storage.
    DigitalStorage,
    /// Time duration.
    Time,
    /// Monetary currency.
    Currency,
}

/// Unit definition with its canonical conversion factor to base SI/canonical unit.
#[derive(Debug, Clone)]
pub struct UnitDefinition {
    /// Canonical display symbol (e.g. "km", "USD", "°C").
    pub symbol: &'static str,
    /// Full human-readable name.
    pub name: &'static str,
    /// Physical or monetary dimension.
    pub dimension: UnitDimension,
    /// Linear multiplier to base unit (for non-temperature units).
    pub factor_to_base: f64,
}

/// Result of a successful unit conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversionResult {
    /// Source amount converted from.
    pub from_amount: f64,
    /// Source unit canonical symbol.
    pub from_unit: String,
    /// Target converted amount.
    pub to_amount: f64,
    /// Target unit canonical symbol.
    pub to_unit: String,
    /// Formatted summary string (e.g. "100 km = 62.1371 mi").
    pub formatted: String,
}

/// Main Unit and Currency conversion engine.
pub struct UnitConverter {
    units: HashMap<String, UnitDefinition>,
}

impl Default for UnitConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl UnitConverter {
    /// Creates a converter initialized with physical SI, Imperial, Digital, and Currency tables.
    pub fn new() -> Self {
        let mut converter = Self {
            units: HashMap::new(),
        };
        converter.register_builtin_units();
        converter
    }

    fn register(&mut self, aliases: &[&str], def: UnitDefinition) {
        for alias in aliases {
            self.units.insert(alias.to_lowercase(), def.clone());
        }
    }

    fn register_builtin_units(&mut self) {
        // Base Length: Meter (m)
        self.register(&["m", "meter", "meters", "metro", "metros"], UnitDefinition {
            symbol: "m",
            name: "Meter",
            dimension: UnitDimension::Length,
            factor_to_base: 1.0,
        });
        self.register(&["km", "kilometer", "kilometers", "quilometro", "quilometros"], UnitDefinition {
            symbol: "km",
            name: "Kilometer",
            dimension: UnitDimension::Length,
            factor_to_base: 1000.0,
        });
        self.register(&["cm", "centimeter", "centimeters", "centimetro"], UnitDefinition {
            symbol: "cm",
            name: "Centimeter",
            dimension: UnitDimension::Length,
            factor_to_base: 0.01,
        });
        self.register(&["mm", "millimeter", "millimeters", "milimetro"], UnitDefinition {
            symbol: "mm",
            name: "Millimeter",
            dimension: UnitDimension::Length,
            factor_to_base: 0.001,
        });
        self.register(&["in", "inch", "inches", "polegada", "polegadas"], UnitDefinition {
            symbol: "in",
            name: "Inch",
            dimension: UnitDimension::Length,
            factor_to_base: 0.0254,
        });
        self.register(&["ft", "foot", "feet", "pe", "pes"], UnitDefinition {
            symbol: "ft",
            name: "Foot",
            dimension: UnitDimension::Length,
            factor_to_base: 0.3048,
        });
        self.register(&["yd", "yard", "yards", "jarda", "jardas"], UnitDefinition {
            symbol: "yd",
            name: "Yard",
            dimension: UnitDimension::Length,
            factor_to_base: 0.9144,
        });
        self.register(&["mi", "mile", "miles", "milha", "milhas"], UnitDefinition {
            symbol: "mi",
            name: "Mile",
            dimension: UnitDimension::Length,
            factor_to_base: 1609.344,
        });

        // Base Mass: Kilogram (kg)
        self.register(&["kg", "kilogram", "kilograms", "quilo", "quilos"], UnitDefinition {
            symbol: "kg",
            name: "Kilogram",
            dimension: UnitDimension::Mass,
            factor_to_base: 1.0,
        });
        self.register(&["g", "gram", "grams", "grama", "gramas"], UnitDefinition {
            symbol: "g",
            name: "Gram",
            dimension: UnitDimension::Mass,
            factor_to_base: 0.001,
        });
        self.register(&["mg", "milligram", "milligrams", "miligrama"], UnitDefinition {
            symbol: "mg",
            name: "Milligram",
            dimension: UnitDimension::Mass,
            factor_to_base: 0.000001,
        });
        self.register(&["lb", "lbs", "pound", "pounds", "libra", "libras"], UnitDefinition {
            symbol: "lb",
            name: "Pound",
            dimension: UnitDimension::Mass,
            factor_to_base: 0.45359237,
        });
        self.register(&["oz", "ounce", "ounces", "onca", "oncas"], UnitDefinition {
            symbol: "oz",
            name: "Ounce",
            dimension: UnitDimension::Mass,
            factor_to_base: 0.028349523125,
        });
        self.register(&["ton", "tonne", "tonnes", "tonelada", "toneladas"], UnitDefinition {
            symbol: "ton",
            name: "Metric Ton",
            dimension: UnitDimension::Mass,
            factor_to_base: 1000.0,
        });

        // Base Temperature: Celsius (°C)
        self.register(&["c", "°c", "celsius"], UnitDefinition {
            symbol: "°C",
            name: "Celsius",
            dimension: UnitDimension::Temperature,
            factor_to_base: 1.0,
        });
        self.register(&["f", "°f", "fahrenheit"], UnitDefinition {
            symbol: "°F",
            name: "Fahrenheit",
            dimension: UnitDimension::Temperature,
            factor_to_base: 1.0,
        });
        self.register(&["k", "kelvin"], UnitDefinition {
            symbol: "K",
            name: "Kelvin",
            dimension: UnitDimension::Temperature,
            factor_to_base: 1.0,
        });

        // Base Digital Storage: Byte (B)
        self.register(&["b", "byte", "bytes"], UnitDefinition {
            symbol: "B",
            name: "Byte",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1.0,
        });
        self.register(&["kb", "kilobyte", "kilobytes"], UnitDefinition {
            symbol: "KB",
            name: "Kilobyte (decimal)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1_000.0,
        });
        self.register(&["mb", "megabyte", "megabytes"], UnitDefinition {
            symbol: "MB",
            name: "Megabyte (decimal)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1_000_000.0,
        });
        self.register(&["gb", "gigabyte", "gigabytes"], UnitDefinition {
            symbol: "GB",
            name: "Gigabyte (decimal)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1_000_000_000.0,
        });
        self.register(&["tb", "terabyte", "terabytes"], UnitDefinition {
            symbol: "TB",
            name: "Terabyte (decimal)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1_000_000_000_000.0,
        });
        self.register(&["kib", "kibibyte"], UnitDefinition {
            symbol: "KiB",
            name: "Kibibyte (binary)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1024.0,
        });
        self.register(&["mib", "mebibyte"], UnitDefinition {
            symbol: "MiB",
            name: "Mebibyte (binary)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1024.0 * 1024.0,
        });
        self.register(&["gib", "gibibyte"], UnitDefinition {
            symbol: "GiB",
            name: "Gibibyte (binary)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1024.0 * 1024.0 * 1024.0,
        });
        self.register(&["tib", "tebibyte"], UnitDefinition {
            symbol: "TiB",
            name: "Tebibyte (binary)",
            dimension: UnitDimension::DigitalStorage,
            factor_to_base: 1024.0 * 1024.0 * 1024.0 * 1024.0,
        });

        // Base Time: Second (s)
        self.register(&["s", "sec", "second", "seconds", "segundo", "segundos"], UnitDefinition {
            symbol: "s",
            name: "Second",
            dimension: UnitDimension::Time,
            factor_to_base: 1.0,
        });
        self.register(&["min", "minute", "minutes", "minuto", "minutos"], UnitDefinition {
            symbol: "min",
            name: "Minute",
            dimension: UnitDimension::Time,
            factor_to_base: 60.0,
        });
        self.register(&["h", "hr", "hour", "hours", "hora", "horas"], UnitDefinition {
            symbol: "h",
            name: "Hour",
            dimension: UnitDimension::Time,
            factor_to_base: 3600.0,
        });
        self.register(&["d", "day", "days", "dia", "dias"], UnitDefinition {
            symbol: "d",
            name: "Day",
            dimension: UnitDimension::Time,
            factor_to_base: 86400.0,
        });
        self.register(&["wk", "week", "weeks", "semana", "semanas"], UnitDefinition {
            symbol: "wk",
            name: "Week",
            dimension: UnitDimension::Time,
            factor_to_base: 604800.0,
        });

        // Base Currency: USD (factor to USD)
        self.register(&["usd", "$", "dollar", "dollars", "dolar"], UnitDefinition {
            symbol: "USD",
            name: "US Dollar",
            dimension: UnitDimension::Currency,
            factor_to_base: 1.0,
        });
        self.register(&["eur", "€", "euro", "euros"], UnitDefinition {
            symbol: "EUR",
            name: "Euro",
            dimension: UnitDimension::Currency,
            factor_to_base: 1.08,
        });
        self.register(&["gbp", "£", "pound sterling"], UnitDefinition {
            symbol: "GBP",
            name: "British Pound",
            dimension: UnitDimension::Currency,
            factor_to_base: 1.28,
        });
        self.register(&["jpy", "¥", "yen"], UnitDefinition {
            symbol: "JPY",
            name: "Japanese Yen",
            dimension: UnitDimension::Currency,
            factor_to_base: 0.0066,
        });
        self.register(&["brl", "r$", "real", "reais"], UnitDefinition {
            symbol: "BRL",
            name: "Brazilian Real",
            dimension: UnitDimension::Currency,
            factor_to_base: 0.18,
        });
        self.register(&["cad", "canadian dollar"], UnitDefinition {
            symbol: "CAD",
            name: "Canadian Dollar",
            dimension: UnitDimension::Currency,
            factor_to_base: 0.73,
        });
        self.register(&["aud", "australian dollar"], UnitDefinition {
            symbol: "AUD",
            name: "Australian Dollar",
            dimension: UnitDimension::Currency,
            factor_to_base: 0.66,
        });
        self.register(&["chf", "swiss franc"], UnitDefinition {
            symbol: "CHF",
            name: "Swiss Franc",
            dimension: UnitDimension::Currency,
            factor_to_base: 1.12,
        });
        self.register(&["cny", "yuan", "renminbi"], UnitDefinition {
            symbol: "CNY",
            name: "Chinese Yuan",
            dimension: UnitDimension::Currency,
            factor_to_base: 0.14,
        });
    }

    /// Attempts to parse and execute a conversion query.
    pub fn convert_query(&self, query: &str) -> Result<ConversionResult, ConvertError> {
        let trimmed = query.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        // Check for formats like:
        // ["100", "km", "to", "mi"]
        // ["100", "km", "in", "mi"]
        // ["100km", "to", "mi"]
        let (amount, from_unit_str, to_unit_str) = match parts.as_slice() {
            [amount_str, from_str, sep, to_str] if *sep == "to" || *sep == "in" || *sep == "=" || *sep == "->" => {
                let amount: f64 = amount_str.parse().map_err(|_| ConvertError::InvalidAmount((*amount_str).to_string()))?;
                (amount, *from_str, *to_str)
            }
            [first, sep, to_str] if *sep == "to" || *sep == "in" || *sep == "=" || *sep == "->" => {
                // Split number from unit in first, e.g. "100km"
                let (amount, from_str) = Self::split_number_and_unit(first)?;
                (amount, from_str, *to_str)
            }
            _ => return Err(ConvertError::InvalidFormat),
        };

        self.convert(amount, from_unit_str, to_unit_str)
    }

    fn split_number_and_unit(token: &str) -> Result<(f64, &str), ConvertError> {
        let first_non_num = token
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .ok_or(ConvertError::InvalidFormat)?;

        let (num_part, unit_part) = token.split_at(first_non_num);
        let amount: f64 = num_part
            .parse()
            .map_err(|_| ConvertError::InvalidAmount(num_part.to_string()))?;

        if unit_part.is_empty() {
            return Err(ConvertError::InvalidFormat);
        }

        Ok((amount, unit_part))
    }

    /// Converts an amount from one unit string to another unit string.
    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Result<ConversionResult, ConvertError> {
        let from_def = self
            .units
            .get(&from.to_lowercase())
            .ok_or_else(|| ConvertError::UnknownUnit(from.to_string()))?;

        let to_def = self
            .units
            .get(&to.to_lowercase())
            .ok_or_else(|| ConvertError::UnknownUnit(to.to_string()))?;

        if from_def.dimension != to_def.dimension {
            return Err(ConvertError::IncompatibleDimensions(
                from.to_string(),
                to.to_string(),
            ));
        }

        let to_amount = if from_def.dimension == UnitDimension::Temperature {
            Self::convert_temperature(amount, from_def.symbol, to_def.symbol)
        } else {
            // General linear conversion: amount * from_factor / to_factor
            (amount * from_def.factor_to_base) / to_def.factor_to_base
        };

        let formatted = format!("{amount:.4} {} = {to_amount:.4} {}", from_def.symbol, to_def.symbol);

        Ok(ConversionResult {
            from_amount: amount,
            from_unit: from_def.symbol.to_string(),
            to_amount,
            to_unit: to_def.symbol.to_string(),
            formatted,
        })
    }

    fn convert_temperature(amount: f64, from_symbol: &str, to_symbol: &str) -> f64 {
        if from_symbol == to_symbol {
            return amount;
        }

        // Convert to Celsius first
        let celsius = match from_symbol {
            "°C" => amount,
            "°F" => (amount - 32.0) * (5.0 / 9.0),
            "K" => amount - 273.15,
            _ => amount,
        };

        // Convert from Celsius to target
        match to_symbol {
            "°C" => celsius,
            "°F" => (celsius * (9.0 / 5.0)) + 32.0,
            "K" => celsius + 273.15,
            _ => celsius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversion() {
        let converter = UnitConverter::new();
        let res = converter.convert_query("100 km to mi").unwrap();
        assert_eq!(res.from_unit, "km");
        assert_eq!(res.to_unit, "mi");
        assert!((res.to_amount - 62.1371).abs() < 0.01);
    }

    #[test]
    fn test_temperature_conversion() {
        let converter = UnitConverter::new();
        let res = converter.convert_query("32 F to C").unwrap();
        assert!((res.to_amount - 0.0).abs() < 1e-6);

        let k_res = converter.convert_query("100 C in K").unwrap();
        assert!((k_res.to_amount - 373.15).abs() < 1e-6);
    }

    #[test]
    fn test_digital_storage_conversion() {
        let converter = UnitConverter::new();
        let res = converter.convert_query("1 GiB to MiB").unwrap();
        assert_eq!(res.to_amount, 1024.0);
    }

    #[test]
    fn test_currency_conversion() {
        let converter = UnitConverter::new();
        let res = converter.convert_query("100 USD to BRL").unwrap();
        assert!((res.to_amount - (100.0 / 0.18)).abs() < 0.1);
    }

    #[test]
    fn test_incompatible_dimensions() {
        let converter = UnitConverter::new();
        assert_eq!(
            converter.convert_query("10 km to kg"),
            Err(ConvertError::IncompatibleDimensions("km".into(), "kg".into()))
        );
    }
}
