use crate::Arghs;
use plotters::prelude::*;
use std::ops::Deref;

const NAME_SUFFIX: &str = "sense_plot.png";

/// Plots graphs
/// Takes frequencies series, power series and temperature series
pub fn plot(
    args: &Arghs,
    freq_series: &[f64],
    power_series: &[f64],
    temp_series: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    let secs_epoch = crate::TIMESTAMP.deref();
    let wxh = args.wxh;

    let power_and_temperatue = format!("power_and_temperatue_{}_{}", secs_epoch, NAME_SUFFIX);
    let power_and_frequency = format!("power_and_frequency_{}_{}", secs_epoch, NAME_SUFFIX);

    let pt_root = BitMapBackend::new(&power_and_temperatue, wxh).into_drawing_area();
    pt_root.fill(&WHITE)?;

    let pf_root = BitMapBackend::new(&power_and_frequency, wxh).into_drawing_area();
    pf_root.fill(&WHITE)?;

    let mut pt_chart = ChartBuilder::on(&pt_root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .right_y_label_area_size(40u32)
        .margin(5u32)
        .caption(&args.title, ("sans-serif", 35.0).into_font())
        .build_cartesian_2d(0usize..power_series.len(), 0f64..100f64)?
        .set_secondary_coord(0usize..power_series.len(), 0f64..110f64);

    let mut pf_chart = ChartBuilder::on(&pf_root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .right_y_label_area_size(40u32)
        .margin(5u32)
        .caption(&args.title, ("sans-serif", 35.0).into_font())
        .build_cartesian_2d(0usize..power_series.len(), 0f64..4800f64)?
        .set_secondary_coord(0usize..power_series.len(), 0f64..110f64);

    pt_chart
        .configure_mesh()
        .disable_x_mesh()
        .y_desc("Temperature (°C)")
        .draw()?;

    pf_chart
        .configure_mesh()
        .disable_x_mesh()
        .y_desc("Clock (MHz)")
        .draw()?;

    pt_chart
        .configure_secondary_axes()
        .y_desc("Power (Watt)")
        .draw()?;

    pf_chart
        .configure_secondary_axes()
        .y_desc("Power (Watt)")
        .draw()?;

    pt_chart
        .draw_series(LineSeries::new(
            (0..temp_series.len()).zip(temp_series.iter().cloned()),
            (&BLUE).stroke_width(2),
        ))?
        .label("y = Tdie (° C)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    pf_chart
        .draw_series(LineSeries::new(
            (0..freq_series.len()).zip(freq_series.iter().cloned()),
            (&BLUE).stroke_width(2),
        ))?
        .label("y = Clock frequency (MHz)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    pt_chart
        .draw_secondary_series(LineSeries::new(
            power_series.iter().cloned().enumerate(),
            (&RED).stroke_width(2),
        ))?
        .label("y = Power(W)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    pf_chart
        .draw_secondary_series(LineSeries::new(
            power_series.iter().cloned().enumerate(),
            (&RED).stroke_width(2),
        ))?
        .label("y = Power(W)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    pt_chart
        .configure_series_labels()
        .background_style(&RGBColor(128, 128, 128))
        .draw()?;

    pf_chart
        .configure_series_labels()
        .background_style(&RGBColor(128, 128, 128))
        .draw()?;

    pt_root.present()?;
    pf_root.present()?;
    Ok(())
}
