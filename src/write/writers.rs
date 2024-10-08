use num_traits::ToBytes;

use crate::parse::{ArrayFlags, DataElement, DataType, NumericData};

use std::io::{Result, Write};

pub fn write_header<W: Write>(w: &mut W, text: &str) -> Result<()> {
    let text_bytes = match text.len() {
        0..=3 => "MATLAB 5.0 MAT-file".as_bytes(),
        4.. => text.as_bytes(),
    };

    if text_bytes.len() > 116 {
        return Err(std::io::Error::other(
            "Header length can't be more than 116",
        ));
    }

    // Write description
    w.write_all(text_bytes)?;
    // Ensure proper padding
    w.write_all(&vec![32; 116 - text_bytes.len()])?;

    // Indicate no subsystem specific data
    w.write_all(&vec![0; 8])?;

    // Set version to 0x0100
    w.write_all(&[0b00000000, 0b00000001])?;

    // Write endiannes flag
    // 'M' = 77, 'I' = 73
    let a = (77u16 << 8) + 73;
    w.write_all(&a.to_ne_bytes())?;

    Ok(())
}

pub fn write_matrix<W: Write>(w: &mut W, data_element: DataElement) -> Result<()> {
    // Calculate size before destructuring
    // This allows us to write the element tag and data sequentially
    // Otherwise we would have to seek in the writer or copy all of
    // the data into a buffer before writing it
    let calculated_size = sizes::data_element(&data_element);
    let padding_size = padding_size(calculated_size);
    let number_of_bytes = match data_element {
        // Number of bytes following including 64bit padding
        DataElement::NumericMatrix(..) => calculated_size + padding_size,
        // Number of bytes following
        _ => calculated_size,
    };

    let DataElement::NumericMatrix(array_flags, dimensions, matrix_name, real_part, imaginary_part) =
        data_element
    else {
        panic!("unsupported");
    };

    // Write MAT-File Data Type
    w.write_all(&(DataType::Matrix as u32).to_ne_bytes())?;

    // Write number of bytes in this data element
    w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

    // Write actual data
    {
        write_sub_element_array_flags(w, array_flags)?;

        write_sub_element_dimensions(w, &dimensions)?;
        write_sub_element_array_name(w, &matrix_name)?;

        let real_data_type = real_part.data_type();
        let real_part_data = real_part.to_ne_bytes();
        write_sub_element_real_part(w, real_data_type, &real_part_data)?;

        if let Some(imaginary_part) = imaginary_part {
            let imaginary_data_type = imaginary_part.data_type();
            let imaginary_part_data = imaginary_part.to_ne_bytes();
            write_sub_element_imaginary_part(w, imaginary_data_type, &imaginary_part_data)?;
        }
    }

    // Ensure 64 bit padding
    w.write_all(&(vec![0; padding_size]))?;

    Ok(())
}

pub fn write_data_element<W: Write>(
    w: &mut W,
    data_type: DataType,
    byte_data: &[u8],
) -> Result<()> {
    // Write MAT-File Data Type
    w.write_all(&(data_type as u32).to_ne_bytes())?;

    let padding_byte_count = padding_size(byte_data.len());

    // Write number of bytes in this data element
    let number_of_bytes = match data_type {
        // Number of bytes following including 64bit padding
        DataType::Matrix => byte_data.len() + padding_byte_count,
        // Number of bytes following
        _ => byte_data.len(),
    };
    w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

    // Write actual data
    w.write_all(byte_data)?;

    // Ensure 64 bit padding
    w.write_all(&(vec![0; padding_byte_count]))?;

    Ok(())
}

pub fn write_sub_element_array_flags<W: Write>(w: &mut W, array_flags: ArrayFlags) -> Result<()> {
    // Sub element data type
    w.write_all(&(DataType::UInt32 as u32).to_ne_bytes())?;
    // Sub element number of bytes
    w.write_all(&(8 as u32).to_ne_bytes())?;

    let class = array_flags.class as u8;
    let flags = ((array_flags.complex as u8) << 3)
        + ((array_flags.global as u8) << 2)
        + ((array_flags.logical as u8) << 1);

    // Figure 1-6
    // The reason why this is endianess dependent is beyond me
    let flags_u32 = (class as u32) + ((flags as u32) << 8);
    w.write_all(&flags_u32.to_ne_bytes())?;

    // This should only matter for spare arrays
    w.write_all(&(array_flags.nzmax as u32).to_ne_bytes())?;

    Ok(())
}

pub fn write_sub_element_dimensions<W: Write>(w: &mut W, dimensions: &[i32]) -> Result<()> {
    assert!(dimensions.len() <= 3);

    // Sub element data type
    w.write_all(&(DataType::Int32 as u32).to_ne_bytes())?;

    // Sub element number of bytes
    let number_of_bytes = dimensions.len() * 4;
    w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

    // Write dimensions
    for dimension in dimensions {
        w.write_all(&dimension.to_ne_bytes())?;
    }

    // Write padding
    w.write_all(&vec![0; padding_size(number_of_bytes)])?;

    Ok(())
}

pub fn write_sub_element_array_name<W: Write>(w: &mut W, array_name: &str) -> Result<()> {
    // Sub element data type
    w.write_all(&(DataType::Int8 as u32).to_ne_bytes())?;

    // Sub element number of bytes
    let array_name = array_name.as_bytes();
    let number_of_bytes = array_name.len();
    w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

    // Write array name
    w.write_all(array_name)?;

    // Write padding
    w.write_all(&vec![0; padding_size(number_of_bytes)])?;

    Ok(())
}

pub fn write_sub_element_real_part<W: Write>(
    w: &mut W,
    data_type: DataType,
    data: &[u8],
) -> Result<()> {
    // !TODO error handling
    let _size_of_data_type = data_type
        .byte_size()
        .expect("Can't use non numerical data type for real part");

    // Sub element data type
    w.write_all(&(data_type as u32).to_ne_bytes())?;

    // Sub element number of bytes
    let number_of_bytes = data.len();
    // assert_eq!(data.len() % size_of_data_type, 0);
    w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

    // Write real part
    w.write_all(data)?;

    // Write padding
    w.write_all(&vec![0; padding_size(number_of_bytes)])?;

    Ok(())
}

// Is there even a difference?
pub fn write_sub_element_imaginary_part<W: Write>(
    w: &mut W,
    data_type: DataType,
    data: &[u8],
) -> Result<()> {
    write_sub_element_real_part(w, data_type, data)
}

impl ToBytes for NumericData {
    type Bytes = Vec<u8>;

    fn to_ne_bytes(&self) -> Vec<u8> {
        // !TODO check soundness of align_to for potential big speed improvement
        match self {
            NumericData::Single(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Double(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
        }
    }

    fn to_be_bytes(&self) -> Vec<u8> {
        // !TODO check soundness of align_to for potential big speed improvement
        match self {
            NumericData::Single(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Double(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        // !TODO check soundness of align_to for potential big speed improvement
        match self {
            NumericData::Single(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Double(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt8(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt16(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt32(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::Int64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
            NumericData::UInt64(vec) => vec.into_iter().flat_map(|&n| n.to_ne_bytes()).collect(),
        }
    }
}

use sizes::padding_size;

pub mod sizes {
    use crate::parse::{DataElement, DataType};

    pub fn data_element(elm: &DataElement) -> usize {
        let DataElement::NumericMatrix(
            _array_flags,
            dimensions,
            matrix_name,
            real_part,
            imaginary_part,
        ) = elm
        else {
            panic!("Size calculation not yet supported for types other than numeric matrix");
        };

        let array_flags_size = 8 + 8;
        let dimensions_size = self::dimensions(dimensions.len());
        let matrix_name_size = name(matrix_name.bytes().len());
        let real_part_size = numeric_subelement(real_part.data_type(), real_part.len());
        let imaginary_part_size = match imaginary_part {
            Some(part) => numeric_subelement(part.data_type(), part.len()),
            None => 0,
        };

        array_flags_size + dimensions_size + matrix_name_size + real_part_size + imaginary_part_size
    }

    pub fn name(byte_count: usize) -> usize {
        let tag_size = 8;
        let padding_size = padding_size(byte_count);

        tag_size + byte_count + padding_size
    }

    pub fn dimensions(dim_count: usize) -> usize {
        let tag_size = 8;
        let number_of_bytes = dim_count * 4;
        let padding_size = padding_size(number_of_bytes);

        tag_size + number_of_bytes + padding_size
    }

    pub fn numeric_subelement(data_type: DataType, len: usize) -> usize {
        let tag_size = 8;
        let number_of_bytes = len
            * data_type
                .byte_size()
                .expect("Unexpected non numeric data type in NumericMatrix");
        let padding_size = padding_size(number_of_bytes);

        tag_size + number_of_bytes + padding_size
    }

    pub fn padding_size(byte_count: usize) -> usize {
        match byte_count % 8 {
            0 => 0,
            n => 8 - n,
        }
    }
}
