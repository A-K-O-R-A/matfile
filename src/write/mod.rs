#[cfg(test)]
mod tests;

mod writers;

use num_traits::ToBytes;

use crate::parse::{ArrayFlags, ArrayType, DataType, NumericData};

use std::io::{Result, Write};

pub struct MatFileWriter<'a, W: Write>(&'a mut W);

impl<'a, W: Write> MatFileWriter<'a, W> {
    pub fn new(w: &'a mut W) -> Result<Self> {
        Self::new_with_description(w, "MATLAB 5.0 MAT-file, Platform: matfile-rs")
    }

    pub fn new_with_description(w: &'a mut W, description: &str) -> Result<Self> {
        let matfile = MatFileWriter(w);

        writers::write_header(matfile.0, description)?;

        Ok(matfile)
    }

    pub fn write_array(
        &mut self,
        array_name: &str,
        real: NumericData,
        imag: Option<NumericData>,
    ) -> Result<()> {
        let complex = imag.is_some();
        if let Some(ref imag) = imag {
            // !TODO Throw error instead
            assert_eq!(real.len(), imag.len())
        }

        let dim = vec![1i32, real.len() as i32];

        // This should avoid expensive reallocations for big arrays
        let estimated_size = {
            use writers::sizes::*;
            dimensions(dim.len())
                + name(array_name.bytes().len())
                + numeric_subelement(real.data_type(), real.len())
                + match imag {
                    Some(ref imag) => numeric_subelement(imag.data_type(), imag.len()),
                    None => 0,
                }
        };

        let mut buf = Vec::with_capacity(estimated_size);
        {
            writers::write_sub_element_array_flags(
                &mut buf,
                ArrayFlags {
                    complex,
                    global: false,
                    logical: false,
                    class: ArrayType::Double,
                    nzmax: 0,
                },
            )?;

            writers::write_sub_element_dimensions(&mut buf, &dim)?;
            writers::write_sub_element_array_name(&mut buf, &array_name)?;

            writers::write_sub_element_real_part(&mut buf, real.data_type(), &real.to_ne_bytes())?;

            if let Some(imag) = imag {
                writers::write_sub_element_imaginary_part(
                    &mut buf,
                    imag.data_type(),
                    &imag.to_ne_bytes(),
                )?;
            }
        }

        writers::write_data_element(self.0, DataType::Matrix, &buf)?;

        self.0.flush()?;

        Ok(())
    }
}
