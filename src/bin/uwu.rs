use matfile::MatFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    let read_path = &args[1];
    let data = std::fs::read(read_path)?;
    let mat_file = crate::MatFile::parse(&data[..])?;
    // println!("{:#?}", mat_file);

    let write_path = &args[2];
    let data = std::fs::File::create(write_path)?;
    mat_file.write(data)?;

    Ok(())
}
