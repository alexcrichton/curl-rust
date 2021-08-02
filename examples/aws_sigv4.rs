use anyhow::Result;

use curl::easy::Easy;

fn main() -> Result<()> {
    let mut handle = Easy::new();
    handle.verbose(true)?;
    handle.url("https://ec2.us-east-1.amazonaws.com/?Action=DescribeRegions&Version=2013-10-15")?;
    handle.aws_sigv4("aws:amz")?;
    handle.username("myAccessKeyId")?;
    handle.password("mySecretAccessKey")?;
    handle.perform()?;
    Ok(())
}
