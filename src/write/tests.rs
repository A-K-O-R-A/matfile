#[cfg(test)]
mod test {
    use super::super::writers;

    use crate::parse::{ArrayFlags, ArrayType, DataElement, DataType, NumericData};

    const REFERENCE: &[u8] = include_bytes!("../../tests/small_matrix.mat");

    #[test]
    fn write_header() {
        let mut buf = Vec::new();

        writers::write_header(
            &mut buf,
            "MATLAB 5.0 MAT-file, Platform: GLNXA64, Created on: Mon Jun 17 17:55:27 2024",
        )
        .expect("Writing into a buffer should not fail");

        // !TODO: This test will fail on big endian systems
        assert_eq!(&buf[0..128], &REFERENCE[0..128]);
    }

    #[test]
    fn write_data_element_tag_matrix() {
        let mut buf = Vec::new();

        // The referecne asset file contains 72 bytes of data
        writers::write_data_element(&mut buf, DataType::Matrix, &[0; 72])
            .expect("Writing into a buffer should not fail");

        let prev_offset = 128;
        let byte_length = 8;
        assert_eq!(
            &buf[0..byte_length],
            &REFERENCE[prev_offset..(prev_offset + byte_length)]
        );
    }

    #[test]
    fn write_sub_element_array_flags() {
        let mut buf = Vec::new();

        // The referecne asset file contains 72 bytes of data
        writers::write_sub_element_array_flags(
            &mut buf,
            ArrayFlags {
                class: ArrayType::Double,
                complex: false,
                global: false,
                logical: false,
                nzmax: 0,
            },
        )
        .expect("Writing into a buffer should not fail");

        let prev_offset = 136;
        let byte_length = 16;
        assert_eq!(
            &buf[0..byte_length],
            &REFERENCE[prev_offset..(prev_offset + byte_length)]
        );
    }

    #[test]
    fn write_sub_element_array_dimensions() {
        let mut buf = Vec::new();

        // The referecne asset file contains 72 bytes of data
        writers::write_sub_element_dimensions(&mut buf, &[1, 3])
            .expect("Writing into a buffer should not fail");

        let prev_offset = 152;
        let byte_length = 16;
        assert_eq!(
            &buf[0..byte_length],
            &REFERENCE[prev_offset..(prev_offset + byte_length)]
        );
    }

    #[test]
    fn write_sub_element_array_name() {
        let mut buf = Vec::new();

        // The referecne asset file contains 72 bytes of data
        writers::write_sub_element_array_name(&mut buf, "abcde")
            .expect("Writing into a buffer should not fail");

        let prev_offset = 168;
        let byte_length = 16;
        assert_eq!(
            &buf[0..byte_length],
            &REFERENCE[prev_offset..(prev_offset + byte_length)]
        );
    }

    #[test]
    fn write_sub_element_real_part() {
        let mut bytes: Vec<u8> = Vec::new();
        for num in [1i32, 2, 21474836] {
            bytes.extend(num.to_ne_bytes());
        }

        let mut buf = Vec::new();

        // The referecne asset file contains 72 bytes of data
        writers::write_sub_element_real_part(&mut buf, DataType::Int32, &bytes)
            .expect("Writing into a buffer should not fail");

        let prev_offset = 184;
        let byte_length = 24;
        assert_eq!(
            &buf[0..byte_length],
            &REFERENCE[prev_offset..(prev_offset + byte_length)]
        );
    }

    #[test]
    fn write_full_file() {
        let mut buf = Vec::new();

        writers::write_header(
            &mut buf,
            "MATLAB 5.0 MAT-file, Platform: GLNXA64, Created on: Mon Jun 17 17:55:27 2024",
        )
        .expect("Writing into a buffer should not fail");

        let mut matrix_data_buf = Vec::new();
        writers::write_sub_element_array_flags(
            &mut matrix_data_buf,
            ArrayFlags {
                complex: false,
                global: false,
                logical: false,
                class: ArrayType::Double,
                nzmax: 0,
            },
        )
        .expect("Writing into a buffer should not fail");

        writers::write_sub_element_dimensions(&mut matrix_data_buf, &[1, 3])
            .expect("Writing into a buffer should not fail");
        writers::write_sub_element_array_name(&mut matrix_data_buf, "abcde")
            .expect("Writing into a buffer should not fail");

        let mut bytes: Vec<u8> = Vec::new();
        for num in [1i32, 2, 21474836] {
            bytes.extend(num.to_ne_bytes());
        }
        writers::write_sub_element_real_part(&mut matrix_data_buf, DataType::Int32, &bytes)
            .expect("Writing into a buffer should not fail");

        writers::write_data_element(&mut buf, DataType::Matrix, &matrix_data_buf)
            .expect("Writing into a buffer should not fail");

        // This helps for better debugging
        for i in 0..REFERENCE.len() {
            assert_eq!(buf[i], REFERENCE[i], "Byte {} should match", i)
        }
        // assert_eq!(buf, REFERENCE);
    }

    #[test]
    fn write_full_file_2() {
        let mut buf = Vec::new();

        writers::write_header(
            &mut buf,
            "MATLAB 5.0 MAT-file, Platform: GLNXA64, Created on: Mon Jun 17 17:55:27 2024",
        )
        .expect("Writing into a buffer should not fail");

        writers::write_matrix(
            &mut buf,
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
                NumericData::Int32(vec![1, 2, 21474836]),
                None,
            ),
        )
        .expect("Writing into a buffer should not fail");

        // This helps for better debugging
        for i in 0..REFERENCE.len() {
            assert_eq!(buf[i], REFERENCE[i], "Byte {} should match", i)
        }
        // assert_eq!(buf, REFERENCE);
    }
}
