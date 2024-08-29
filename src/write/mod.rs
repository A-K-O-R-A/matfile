#[cfg(test)]
mod tests;

mod writers;

use num_traits::ToBytes;

use crate::parse::{ArrayFlags, ArrayType, DataType};

use std::io::{Result, Write};

pub struct MatFileWriter<'a, W: Write>(&'a mut W);

impl<'a, W: Write> std::io::Write for MatFileWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

/*
impl<'a, W: Write> std::ops::Deref for MatFileWriter<'a, W> {
    type Target = &'a mut W;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, W: Write> std::ops::DerefMut for MatFileWriter<'a, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
*/

impl<'a, W: Write> MatFileWriter<'a, W> {
    pub fn new(w: &'a mut W) -> Result<Self> {
        Self::new_with_description(w, "MATLAB 5.0 MAT-file, Platform: matfile-rs")
    }

    pub fn new_with_description(w: &'a mut W, description: &str) -> Result<Self> {
        let mut matfile = MatFileWriter(w);

        writers::write_header(&mut matfile, description)?;

        Ok(matfile)
    }

    pub fn example(w: &'a mut W, array_name: &str, nums: &[i32]) -> Result<Self> {
        let mut matfile = MatFileWriter(w);

        writers::write_header(
            &mut matfile,
            "MATLAB 5.0 MAT-file, Platform: GLNXA64, Created on: Mon Jun 17 17:55:27 2024",
        )?;

        let mut buf = Vec::new();
        writers::write_sub_element_array_flags(
            &mut buf,
            ArrayFlags {
                complex: false,
                global: false,
                logical: false,
                class: ArrayType::Double,
                nzmax: 0,
            },
        )?;

        writers::write_sub_element_dimensions(&mut buf, &[1, 3])?;
        writers::write_sub_element_array_name(&mut buf, array_name)?;
        let mut bytes: Vec<u8> = Vec::new();
        for num in nums {
            bytes.extend(num.to_ne_bytes());
        }
        writers::write_sub_element_real_part(&mut buf, DataType::Int32, &bytes)?;

        writers::write_data_element(&mut matfile, DataType::Matrix, &buf)?;

        matfile.flush()?;

        Ok(matfile)
    }
}
