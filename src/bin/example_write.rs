use std::io::Write;

use matfile::{parse::NumericData, write::MatFileWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = std::fs::File::create("output.mat")?;
    let mut writer = MatFileWriter::new(&mut f)?;

    for i in 0..10 {
        writer.write_array(
            &format!("array_{i}"),
            NumericData::Int32(vec![i as _; i as usize]),
            Some(NumericData::Int32(vec![i as _; i as usize])),
        )?;
    }

    f.flush()?;

    Ok(())
}
