use bsnext_input::{InputCreation, InputWriter};
use bsnext_md::md_fs::MdFs;
use bsnext_md::md_writer::MdWriter;

#[test]
fn test_input_to_str() -> anyhow::Result<()> {
    let input_str = include_str!("../../../examples/markdown/single.md");
    let input = MdFs::from_input_str(&input_str, &Default::default()).expect("unwrap");
    let output = MdWriter.input_to_str(&input);
    println!("{}", output);
    let input = MdFs::from_input_str(&output, &Default::default()).expect("unwrapped 2");
    println!("{:?}", input);
    assert_eq!(input.servers.len(), 1);
    assert_eq!(input.servers.first().unwrap().routes.len(), 2);
    Ok(())
}
