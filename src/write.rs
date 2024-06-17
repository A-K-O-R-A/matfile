use num_traits::{ToBytes, ToPrimitive};

use crate::{
    parse::{ArrayFlags, ArrayType, DataElement, DataType, Header},
    MatFile,
};

use std::io::{Result, Write};

impl MatFile {
    pub fn write<W: Write>(&self, w: W) -> Result<()> {
        self.write_header(
            w,
            Header {
                text: "_".to_owned(),
                is_little_endian: false,
            },
        )?;
        Ok(())
    }

    fn write_header<W: Write>(&self, mut w: W, mut header: Header) -> Result<()> {
        if header.text.len() < 4 {
            header.text = "MATLAB 5.0 MAT-file".to_owned();
        }

        let text_bytes = header.text.as_bytes();
        // Wriute description
        w.write_all(text_bytes)?;
        // Ensure proper padding
        w.write_all(&vec![0; 116 - text_bytes.len()])?;

        // Indicate no subsystem specific data
        w.write_all(&vec![0; 8])?;

        // Set version to 0x0100
        w.write_all(&[0x01, 0x00])?;

        // Write endiannes flag
        // 'M' = 77, 'I' = 73
        let a = (77u16 << 8) + 73;
        w.write_all(&a.to_ne_bytes())?;

        Ok(())
    }

    fn write_data_element<W: Write>(
        &self,
        mut w: W,
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

    fn write_sub_element_array_flags<W: Write>(
        &self,
        mut w: W,
        flags: ArrayFlags,
        array_type: ArrayType,
    ) -> Result<()> {
        // Sub element data type
        w.write_all(&(DataType::UInt32 as u32).to_ne_bytes())?;
        // Sub element number of bytes
        w.write_all(&(8 as u32).to_ne_bytes())?;

        let flags = ((flags.complex as u8) << 3)
            + ((flags.global as u8) << 2)
            + ((flags.logical as u8) << 1);

        let class = array_type as u8;

        // Figure 1-6
        w.write_all(&[0, 0, flags, class, 0, 0, 0, 0])?;

        Ok(())
    }

    fn write_sub_element_dimension<W: Write>(&self, mut w: W, dimensions: &[i32]) -> Result<()> {
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

    fn write_sub_element_array_name<W: Write>(&self, mut w: W, array_name: &str) -> Result<()> {
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

    fn write_sub_element_real_part<W: Write>(
        &self,
        mut w: W,
        data_type: DataType,
        data: &[u8],
    ) -> Result<()> {
        // !TODO error handling
        let size_of_data_type = data_type
            .byte_size()
            .expect("Can't use non numerical data type for real part");

        // Sub element data type
        w.write_all(&(data_type as u32).to_ne_bytes())?;

        // Sub element number of bytes
        let number_of_bytes = data.len();
        assert_eq!(data.len() % size_of_data_type, 0);
        w.write_all(&(number_of_bytes as u32).to_ne_bytes())?;

        // Write real part
        w.write_all(data)?;

        // Write padding
        w.write_all(&vec![0; padding_size(number_of_bytes)])?;

        Ok(())
    }

    // IS there a difference?
    fn write_sub_element_imaginary_part<W: Write>(
        &self,
        mut w: W,
        data_type: DataType,
        data: &[u8],
    ) -> Result<()> {
        self.write_sub_element_real_part(w, data_type, data)
    }
}

fn padding_size(byte_count: usize) -> usize {
    match byte_count % 8 {
        0 => 0,
        n => 8 - n,
    }
}
