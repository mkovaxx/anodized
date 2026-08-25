//! Asserts the generated tables in the skill are current.

use skill_tools::{Result, generate};

#[test]
fn generated_regions_are_up_to_date() -> Result<()> {
    let stale = generate::check()?;
    assert!(
        stale.is_empty(),
        "stale generated regions {stale:?}; run \
         `cargo run -p skill-tools --bin skill-gen -- --write`"
    );
    Ok(())
}

#[test]
fn writing_is_idempotent() -> Result<()> {
    for (file, _, once) in generate::rendered_files()? {
        let mut twice = once.clone();
        for (region_file, id) in generate::REGIONS {
            if *region_file == file {
                twice = generate::splice(&twice, id, &generate::render(id)?)?;
            }
        }
        assert_eq!(once, twice, "{file}: a second write is not a no-op");
    }
    Ok(())
}

#[test]
fn every_region_is_declared_exactly_once() -> Result<()> {
    for (file, id) in generate::REGIONS {
        let text = skill_tools::read_skill_file(file)?;
        let open = format!("<!-- anodized:generated:{id} -->");
        let close = format!("<!-- /anodized:generated:{id} -->");
        assert_eq!(
            text.matches(&open).count(),
            1,
            "{file}: `{id}` opening marker"
        );
        assert_eq!(
            text.matches(&close).count(),
            1,
            "{file}: `{id}` closing marker"
        );
    }
    Ok(())
}
