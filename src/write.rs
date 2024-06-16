use num_traits::{ToBytes, ToPrimitive};

use crate::{
    parse::{DataElement, DataType, Header},
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

        let padding_byte_count = match byte_data.len() % 8 {
            0 => 0,
            n => 8 - n,
        };

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
}
