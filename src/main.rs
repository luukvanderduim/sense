mod acquire;
mod plot;

use argh::FromArgs;
use csv::Writer;
use once_cell::sync::Lazy;
use std::io::{stdout, Read, StdoutLock, Write};
use std::time::SystemTime;
use std::{error::Error, thread::sleep, time::Duration};
use termion::{
    async_stdin, clear, cursor,
    cursor::Goto,
    raw::{IntoRawMode, RawTerminal},
};

/// Determine duration since epoch, once, lazily.
/// When `Lazy` is stabilized in `std::sync::Lazy` this can move over to that.
pub static TIMESTAMP: Lazy<String> = Lazy::new(|| {
    let secs_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("failed to get duration since epoch");
    secs_epoch.as_secs().to_string()
});

#[derive(FromArgs)]
/// Simple tool to sample and plot power consumption, average frequency and cpu die temperatures over time.
pub struct Arghs {
    /// optional sample interval in milliseconds (defaults to 1000)
    #[argh(option, short = 'i', default = "1000")]
    interval: u64,

    /// optionally save data series (defaults to `true`)
    #[argh(option, short = 'w', default = "true")]
    write_data: bool,

    /// optional title (e.g. a condition for the run)
    #[argh(
        option,
        short = 't',
        default = "String::from(\"thermals / avg. core clocks under load\")"
    )]
    title: String,

    /// optional image size dimensions WxH (1024x768)
    #[argh(option, from_str_fn(into_plot_dimensions), default = "(1280, 768)")]
    wxh: (u32, u32),
}

// Helper function for parsing plot dimensions from command line arguments.
fn into_plot_dimensions(dim: &str) -> Result<(u32, u32), String> {
    let (w, h) = dim
        .split_once('x')
        .ok_or("dimensions do not parse, no delimiter?")?;
    let w = w.parse::<u32>().map_err(|e| e.to_string())?;
    let h = h.parse::<u32>().map_err(|e| e.to_string())?;
    Ok((w, h))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Arghs = argh::from_env();
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode()?;
    let mut stdin = async_stdin().bytes();

    // Preallocated storage for our sampled data.
    let mut freq_series: Vec<f64> = Vec::with_capacity(2048);
    let mut pwr_series: Vec<f64> = Vec::with_capacity(2048);
    let mut temp_series: Vec<f64> = Vec::with_capacity(2048);

    writeln!(
        stdout,
        "{}{}Press 'q' to stop sampling, write plots and quit.",
        clear::All,
        Goto(1, 1)
    )?;

    // Get sensors' features TODO rename function
    let (temp_feat, power_feat) = acquire::get_temp_and_power()?;

    loop {
        write!(stdout, "{}{}", cursor::Goto(1, 2), clear::AfterCursor)?;

        let power = acquire::get_current_power(&power_feat)?;
        let temp = acquire::get_die_temp(&temp_feat)?;
        let f_avg = acquire::get_avg_freq();

        // Store plot-data
        pwr_series.push(power);
        temp_series.push(temp);
        freq_series.push(f_avg);

        // Show current values
        print_values(&mut stdout, temp, power, f_avg)?;
        stdout.flush()?;

        // Do we quit?
        if let Some(Ok(b'q')) = stdin.next() {
            break;
        }

        // Gather, sleep, repeat
        sleep(Duration::from_millis(args.interval));
    }

    write!(stdout, "{}{}Saving plots..", Goto(1, 8), clear::AfterCursor)?;
    plot::plot(&args, &freq_series, &pwr_series, &temp_series)?;

    if args.write_data {
        write!(stdout, "{}{}Saving csv..", Goto(1, 10), clear::AfterCursor)?;
        let filename_csv = format!("data_series_{}.csv", *TIMESTAMP);
        let mut wtr = Writer::from_path(&filename_csv)?;

        // Write column names
        wtr.serialize(["Power", "Clock", "Temperature"])?;

        //  A sample of each quantity together makes a sample record.
        // This loops iterates over these samples and writes them to the CSV.
        for sample in pwr_series
            .iter()
            .zip(freq_series.iter())
            .zip(temp_series.iter())
        {
            wtr.serialize(sample)?;
        }
        wtr.flush()?;
    }

    write!(stdout, " done!{}", Goto(1, 12))?;
    stdout.flush()?;
    Ok(())
}

fn print_values(
    stdout: &mut RawTerminal<StdoutLock>,
    temp: f64,
    power: f64,
    freq: f64,
) -> std::io::Result<()> {
    write!(stdout, "{}Avg. CPU frequency: {:0.2} MHz", Goto(1, 3), freq)?;
    write!(stdout, "{}CPU die temp: {:0.2} °C", Goto(1, 4), temp)?;
    write!(stdout, "{}CPU power: {: >#4.2} Watt", Goto(1, 5), power)?;
    Ok(())
}
