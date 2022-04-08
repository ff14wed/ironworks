use ironworks::{
	excel::{Excel, FfxivSqpackResource, Language},
	sqpack::{FfxivFsResource, SqPack},
};
use ironworks_schema_saint_coinach::SaintCoinachSchema;

fn main() -> anyhow::Result<()> {
	env_logger::init();

	iw_test()?;
	// stc_test()?;

	Ok(())
}

#[allow(dead_code)]
fn iw_test() -> anyhow::Result<()> {
	let sqpack_resource = FfxivFsResource::search().unwrap();
	let sqpack = SqPack::new(sqpack_resource);

	let resource = FfxivSqpackResource::new(&sqpack);
	// let excel = Excel::new(resource);
	let excel = Excel::with().language(Language::German).build(resource);

	let sheet = excel.sheet("CompanionTransient")?;
	let row = sheet.with().language(Language::English).row(101)?;
	let field = row.field(4)?;
	println!("{field:?}");

	let row = sheet.row(102)?;
	let field = row.field(4)?;
	println!("{field:?}");

	let sheet = excel.sheet("Behavior")?;
	let row = sheet.subrow(30016, 3)?;
	let field = row.field(4)?;
	println!("{field:?}");

	Ok(())
}

#[allow(dead_code)]
fn stc_test() -> anyhow::Result<()> {
	let schema = SaintCoinachSchema::new().unwrap();
	// let version = schema.version("69caa7e14fed1caaeb2089fad484c25e491d3c37").unwrap();
	// let version = schema.version("69caa7e14fed1caaeb2089").unwrap();
	// let version = schema.version("refs/tags/69caa7e").unwrap();
	let version = schema.version("HEAD").unwrap();
	// let version = schema.version("master").unwrap();

	// let schema = version.schema("RelicNote").unwrap();
	// let schema = version.schema("ArrayEventHandler").unwrap();
	// let schema = version.schema("PvPActionSort").unwrap();
	let schema = version.schema("Item").unwrap();

	println!("schema: {:#?}", schema);

	Ok(())
}
