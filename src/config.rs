use std::str::FromStr;


#[derive(Debug)]
pub enum Error
{
    ExpectedValue{argument: String},
    ExclusiveArguments{first: String, second: String},
    NumberParse(String)
}

pub struct Config
{
    pub log_scale: Option<f64>,
    pub min_avg: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub running_avg: Option<u32>,
    pub plot_line: bool,
    pub paths: Vec<String>
}

impl Config
{
    pub fn parse(mut args: impl Iterator<Item=String>) -> Result<Self, Error>
    {
        let mut log_scale = None;
        let mut min_avg = None;
        let mut min_height = None;
        let mut max_height = None;
        let mut running_avg = None;
        let mut plot_line = false;
        let mut paths = Vec::new();

        while let Some(arg) = args.next()
        {
            match arg.as_ref()
            {
                "-l" | "--log" =>
                {
                    log_scale = Some(Self::parse_number(&mut args, arg)?);
                },
                "--min-avg" =>
                {
                    min_avg = Some(Self::parse_number(&mut args, arg)?);
                },
                "-m" | "--min" =>
                {
                    min_height = Some(Self::parse_number(&mut args, arg)?);
                },
                "-M" | "--max" =>
                {
                    max_height = Some(Self::parse_number(&mut args, arg)?);
                },
                "-r" | "--running-avg" =>
                {
                    running_avg = Some(Self::parse_number(&mut args, arg)?);
                },
                "-L" | "--line" =>
                {
                    plot_line = true;
                },
                _ =>
                {
                    paths.push(arg);
                }
            }
        }
        
        if min_avg.is_some() && min_height.is_some()
        {
            return Err(Error::ExclusiveArguments{
                first: "--min-avg".to_owned(),
                second: "--min".to_owned()
            });
        }

        Ok(Self{
            log_scale,
            min_avg,
            min_height,
            max_height,
            running_avg,
            plot_line,
            paths
        })
    }

    fn parse_number<T: FromStr>(
        mut args: impl Iterator<Item=String>,
        arg: String
    ) -> Result<T, Error>
    {
        let value = args.next().ok_or_else(|| Error::ExpectedValue{argument: arg})?;
        
        value.parse().map_err(|_| Error::NumberParse(value))
    }
}
