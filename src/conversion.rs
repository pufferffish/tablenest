use crate::io_utils::{create_reader, create_writer};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use byteorder::WriteBytesExt;
use read_from::{ReadFrom, WriteTo};
use crate::dnt::{Column, Header, WriteCell, FLOAT32, FLOAT64, INT32, LPNNTS, UINT32, UINT8};

pub fn convert_to_tsv(input_path: &PathBuf, output_path: &PathBuf) -> io::Result<()> {
    let start = Instant::now();

    let mut reader = create_reader(input_path)?;
    let mut writer = create_writer(output_path)?;
    let tab: u8 = 0x09;
    let new_line: u8 = 0x0a;

    // header
    let header = Header::read_from(&mut reader)?;
    println!("Rows: {:?}, Cols: {:?}", header.row_count, header.column_count);

    // columns
    let column_count = header.column_count.0 as usize + 1;
    let mut columns: Vec<Column> = Vec::with_capacity(column_count);
    columns.push(Column {
        name: LPNNTS("_RowID|3".to_string()),
        data_type: UINT8(2), // UINT32
    });
    writer.write("_RowID|3".as_bytes())?;
    for _ in 1..column_count {
        writer.write_u8(tab)?;
        let col = Column::read_from(&mut reader)?;
        writer.write(col.name.0.as_bytes())?;
        columns.push(col);
    }
    writer.write_u8(new_line)?;

    // rows
    let row_count = header.row_count.0 as usize;
    for _ in 0..row_count {
        for i in 0..column_count {
            if i > 0 {
                writer.write_u8(tab)?;
            }
            match columns[i].data_type.0 {
                1 => LPNNTS::read_from(&mut reader)?.write_to(&mut writer)?,
                2 => INT32::read_from(&mut reader)?.write_to(&mut writer)?,
                3 => UINT32::read_from(&mut reader)?.write_to(&mut writer)?,
                4 => FLOAT32::read_from(&mut reader)?.write_to(&mut writer)?,
                5 => FLOAT32::read_from(&mut reader)?.write_to(&mut writer)?,
                6 => FLOAT64::read_from(&mut reader)?.write_to(&mut writer)?,
                x => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid column type: {}", x))),
            };
        }
        writer.write_u8(new_line)?;
    }

    println!("Conversion completed in {:?}", start.elapsed());
    Ok(())
}


#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use super::*;

    #[test]
    fn test_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf().join("skillleveltable_rune.tsv");
        let test_path = PathBuf::from("./test/skillleveltable_rune.dnt");
        println!("Test output: {:?}", temp_path);
        convert_to_tsv(&test_path, &temp_path).unwrap();
        let digest = sha256::try_digest(temp_path).unwrap();
        assert_eq!(digest, "7ec02650c896da60e172b8e53539eea2bda67cd7b23002d8c954a4d4fdece0b7");
    }
}