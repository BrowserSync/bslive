use bsnext_input::InputWriter;
use bsnext_md::md_to_input;
use bsnext_md::md_writer::MdWriter;

#[test]
fn test_input_to_str() -> anyhow::Result<()> {
    let input_str = include_str!("../../../examples/markdown/single.md");
    let input = md_to_input(&input_str, &Default::default()).expect("unwrap");
    let output = MdWriter.input_to_str(&input);
    println!("{}", output);
    let input = md_to_input(&output, &Default::default()).expect("unwrapped 2");
    println!("{:#?}", input);
    assert_eq!(input.servers.len(), 1);
    assert_eq!(input.servers.first().unwrap().routes.len(), 2);
    Ok(())
}
