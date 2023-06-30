use sensors::{Chip, Feature, LibsensorsError, Sensors};
use std::error::Error;

/// Get current temperature
pub fn get_die_temp(t_die: &Feature) -> Result<f64, sensors::LibsensorsError> {
    if let Some(t_die_temp) =
        t_die.get_subfeature(sensors::SubfeatureType::SENSORS_SUBFEATURE_TEMP_INPUT)
    {
        Ok(t_die_temp.get_value()?)
    } else {
        Err(LibsensorsError::Unknown)
    }
}

/// Current power consumption.
pub fn get_current_power(p_core: &Feature) -> Result<f64, sensors::LibsensorsError> {
    if let Some(power) =
        p_core.get_subfeature(sensors::SubfeatureType::SENSORS_SUBFEATURE_POWER_INPUT)
    {
        Ok(power.get_value()?)
    } else {
        Err(LibsensorsError::Unknown)
    }
}

/// Returns current average frequency across all cores in MHz.
pub fn get_avg_freq() -> f64 {
    let freqvec = cpu_freq::get();
    let freqvec: Vec<f64> = freqvec.into_iter().map(|f| f.cur.unwrap() as f64).collect();
    freqvec.iter().sum::<f64>() / freqvec.len() as f64
}

/// Gets sensors' temp and power chip features.
/// TODO: separate these or refactor away..
pub fn get_temp_and_power() -> Result<(Feature, Feature), Box<dyn Error>> {
    let sensors = Sensors::new();

    // This relies on [zenpower3](https://github.com/Ta180m/zenpower3)
    // naming.
    // Not sure what the kernel driver, feature or subfeatures will be called once temperature support is added to mainline.
    // TODO: Find out name of kernel driver in later 5.13>= kernels.
    let zenchip = || -> Result<Chip, Box<dyn Error>> {
        Ok(sensors
            .into_iter()
            .find(|chip| chip.get_name().unwrap().starts_with("zen"))
            .ok_or("chip with name 'zenpower' not found")?)
    };

    // Tdie feature
    let t_die: Feature = zenchip()?
        .into_iter()
        .find(|feat| feat.get_label().unwrap().starts_with("Tdie"))
        .ok_or("chip feature  'Tdie' not found")?;

    // SVI2_P_Core feature
    let svi2_p_core: Feature = zenchip()?
        .into_iter()
        .find(|feat| feat.get_label().unwrap().starts_with("SVI2_P_Core"))
        .ok_or("chip feature  'Tdie' not found")?;

    Ok((t_die, svi2_p_core))
}
