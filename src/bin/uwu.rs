use std::io::Write;

use matfile::{
    parse::{ArrayFlags, ArrayType, DataElement, NumericData},
    write::MatFileWriter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    /*
    let read_path = &args[1];
    let data = std::fs::read(read_path)?;
    let mat_file = MatFile::parse(&data[..])?;
    // println!("{:#?}", mat_file);
     */

    let write_path = &args[2];

    let mut f = std::fs::File::create(write_path)?;
    let writer = MatFileWriter;
    writer.write_header(&mut f, "Nahhh bro")?;
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
            vec![1, 3],
            "abcde".to_owned(),
            NumericData::Int32(vec![10, 20, 30]),
            None,
        ),
    )?;

    f.flush()?;

    Ok(())
}
