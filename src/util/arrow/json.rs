use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::json::ArrayWriter;

pub async fn convert_to_json(arrow: &Vec<RecordBatch>) -> Result<Vec<u8>, ArrowError> {
    let mut buf = Vec::with_capacity(1024);
    let mut writer = ArrayWriter::new(&mut buf);
    for batch in arrow {
        writer.write(&batch)?;
    }
    writer.finish()?;

    Ok(buf)
}
