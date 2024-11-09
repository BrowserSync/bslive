use bsnext_md::{input_to_str, md_to_input};

#[test]
fn test_input_to_str() -> anyhow::Result<()> {
    let input_str = include_str!("../../../examples/md-single/md-single.md");
    let input = md_to_input(&input_str).expect("unwrap");
    let output = input_to_str(&input);
    println!("{}", output);
    let input = md_to_input(&output).expect("unwrapped 2");
    println!("{:#?}", input);
    assert_eq!(input.servers.len(), 1);
    assert_eq!(input.servers.first().unwrap().routes.len(), 2);
    Ok(())
}
