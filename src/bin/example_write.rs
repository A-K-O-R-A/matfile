use std::io::Write;

use matfile::{
    parse::{ArrayFlags, ArrayType, DataElement, NumericData},
    write::MatFileWriter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = std::fs::File::create("output.mat")?;
    let writer = MatFileWriter::new_with_description(&mut f, "MATFILE created by matfile")?;

    for i in 0..10 {
        writer.write_matrix(
            &mut f,
            DataElement::NumericMatrix(
                ArrayFlags {
                    complex: false,
                    global: false,
                    logical: false,
                    class: ArrayType::Double,
                    nzmax: 0,
                },
                vec![1, i],
                format!("array_{i}"),
                NumericData::Int32(vec![i; i as usize]),
                None,
            ),
        )?;
    }

    f.flush()?;

    Ok(())
}
