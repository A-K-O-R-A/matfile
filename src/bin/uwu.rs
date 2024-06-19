use matfile::{write::MatFileWriter, MatFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    let read_path = &args[1];
    let data = std::fs::read(read_path)?;
    let mat_file = MatFile::parse(&data[..])?;
    // println!("{:#?}", mat_file);

    let write_path = &args[2];

    let mut data = std::fs::File::create(write_path)?;
    let writer = MatFileWriter;
    writer.write(&mut data, "abcde", &[1, 2, 21474836])?;

    Ok(())
}
